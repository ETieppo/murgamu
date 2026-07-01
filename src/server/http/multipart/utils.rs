use super::MurFormField;
use super::MurMultipart;
use super::MurMultipartConfig;
use super::MurUploadedFile;
use crate::server::error::MurError;
use hyper::body::Bytes;
use std::path::Path;

pub struct MurMultipartUtils;

impl MurMultipartUtils {
	pub fn parse_boundary(content_type: &str) -> Result<String, MurError> {
		if !content_type.to_lowercase().contains("multipart/form-data") {
			return Err(MurError::BadRequest(
				"Content-Type must be multipart/form-data".into(),
			));
		}

		for part in content_type.split(';') {
			let part = part.trim();
			if let Some(boundary) = part.strip_prefix("boundary=") {
				let boundary = boundary.trim_matches('"').trim_matches('\'');
				if boundary.is_empty() {
					return Err(MurError::BadRequest("Empty boundary".into()));
				}
				return Ok(boundary.to_string());
			}
		}

		Err(MurError::BadRequest(
			"Missing boundary in Content-Type".into(),
		))
	}

	pub fn parse_multipart_body(
		body: &Bytes,
		boundary: &str,
		config: &MurMultipartConfig,
	) -> Result<MurMultipart, MurError> {
		let mut multipart = MurMultipart::empty();
		let delimiter = format!("--{}", boundary);
		let parts = Self::split_on_delimiter(body, delimiter.as_bytes());
		let mut file_count = 0;
		let mut field_count = 0;

		for part in parts.iter().skip(1) {
			let part = part
				.strip_prefix(b"\r\n")
				.or_else(|| part.strip_prefix(b"\n"))
				.unwrap_or(part);
			let part = part
				.strip_suffix(b"\r\n")
				.or_else(|| part.strip_suffix(b"\n"))
				.unwrap_or(part);

			if part.is_empty() || part.starts_with(b"--") {
				continue;
			}

			field_count += 1;
			if field_count > config.max_fields {
				return Err(MurError::BadRequest(format!(
					"Too many fields (max: {})",
					config.max_fields
				)));
			}

			let field = Self::parse_part_from_bytes(part, config)?;

			match &field {
				MurFormField::Text { name, value } => {
					multipart
						.text_fields
						.entry(name.clone())
						.or_default()
						.push(value.clone());
				}
				MurFormField::File(file) => {
					file_count += 1;
					if file_count > config.max_files {
						return Err(MurError::BadRequest(format!(
							"Too many files (max: {})",
							config.max_files
						)));
					}

					if file.size() > config.max_file_size {
						return Err(MurError::BadRequest(format!(
							"File '{}' exceeds maximum size of {} bytes",
							file.filename(),
							config.max_file_size
						)));
					}

					if !config.allowed_extensions.is_empty() {
						if let Some(ext) = file.extension() {
							if !config
								.allowed_extensions
								.iter()
								.any(|e| e.eq_ignore_ascii_case(ext))
							{
								return Err(MurError::BadRequest(format!(
									"File extension '{}' is not allowed",
									ext
								)));
							}
						} else {
							return Err(MurError::BadRequest(
								"Files must have an extension".into(),
							));
						}
					}

					if !config.allowed_mime_types.is_empty()
						&& !config
							.allowed_mime_types
							.iter()
							.any(|t| t.eq_ignore_ascii_case(file.content_type()))
					{
						return Err(MurError::BadRequest(format!(
							"Content type '{}' is not allowed",
							file.content_type()
						)));
					}

					multipart.total_file_size += file.size();
					if multipart.total_file_size > config.max_body_size {
						return Err(MurError::BadRequest(format!(
							"Total upload size exceeds maximum of {} bytes",
							config.max_body_size
						)));
					}
					multipart
						.file_fields
						.entry(file.field_name.clone())
						.or_default()
						.push(file.clone());
				}
			}

			multipart.fields.push(field);
		}

		Ok(multipart)
	}

	fn split_on_delimiter<'a>(data: &'a [u8], delimiter: &[u8]) -> Vec<&'a [u8]> {
		let mut parts = Vec::new();
		let mut start = 0;
		while let Some(pos) = Self::find_in_bytes(&data[start..], delimiter) {
			parts.push(&data[start..start + pos]);
			start += pos + delimiter.len();
		}
		parts.push(&data[start..]);
		parts
	}

	fn find_in_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
		if needle.is_empty() {
			return Some(0);
		}
		haystack.windows(needle.len()).position(|w| w == needle)
	}

	fn parse_part_from_bytes(
		part: &[u8],
		config: &MurMultipartConfig,
	) -> Result<MurFormField, MurError> {
		let (headers_bytes, body_bytes) = if let Some(pos) = Self::find_in_bytes(part, b"\r\n\r\n")
		{
			(&part[..pos], &part[pos + 4..])
		} else if let Some(pos) = Self::find_in_bytes(part, b"\n\n") {
			(&part[..pos], &part[pos + 2..])
		} else {
			return Err(MurError::BadRequest("Invalid multipart part format".into()));
		};

		let headers_str = std::str::from_utf8(headers_bytes)
			.map_err(|_| MurError::BadRequest("Non-ASCII characters in part headers".into()))?;

		let mut name = None;
		let mut filename = None;
		let mut content_type = String::from("text/plain");

		for line in headers_str.lines() {
			let line = line.trim();
			if line.is_empty() {
				continue;
			}
			if let Some(value) = line.strip_prefix("Content-Disposition:") {
				let value = value.trim();
				if let Some(name_match) = Self::extract_header_param(value, "name") {
					if name_match.len() > config.max_field_name_length {
						return Err(MurError::BadRequest(format!(
							"Field name exceeds maximum length of {}",
							config.max_field_name_length
						)));
					}
					name = Some(name_match);
				}
				if let Some(filename_match) = Self::extract_header_param(value, "filename") {
					filename = Some(filename_match);
				}
			} else if let Some(value) = line.strip_prefix("Content-Type:") {
				content_type = value.trim().to_string();
			}
		}

		let name = name.ok_or_else(|| MurError::BadRequest("Missing field name in part".into()))?;

		if let Some(filename) = filename {
			Ok(MurFormField::File(MurUploadedFile::new(
				filename,
				content_type,
				Bytes::copy_from_slice(body_bytes),
				name,
			)))
		} else {
			let value = std::str::from_utf8(body_bytes)
				.map_err(|_| MurError::BadRequest("Text field contains non-UTF-8 data".into()))?
				.to_string();
			Ok(MurFormField::Text { name, value })
		}
	}

	pub fn parse_multipart_part(
		part: &str,
		config: &MurMultipartConfig,
	) -> Result<MurFormField, MurError> {
		Self::parse_part_from_bytes(part.as_bytes(), config)
	}

	pub fn extract_header_param(header: &str, param: &str) -> Option<String> {
		let pattern = format!("{}=", param);

		for part in header.split(';') {
			let part = part.trim();
			if let Some(value) = part.strip_prefix(&pattern) {
				let value = value.trim_matches('"').trim_matches('\'');
				return Some(value.to_string());
			}
		}

		None
	}

	pub fn sanitize_filename(filename: &str) -> String {
		let filename = filename
			.chars()
			.filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_' || *c == ' ')
			.collect::<String>();
		let filename = filename.trim_matches(|c| c == '.' || c == ' ');
		let filename = if filename.len() > 255 {
			&filename[..255]
		} else {
			filename
		};

		if filename.is_empty() {
			"unnamed_file".to_string()
		} else {
			filename.to_string()
		}
	}

	pub fn extract_extension(filename: &str) -> Option<String> {
		Path::new(filename)
			.extension()
			.and_then(|ext| ext.to_str())
			.map(|ext| ext.to_lowercase())
	}

	pub fn rand_u32() -> u32 {
		use std::collections::hash_map::RandomState;
		use std::hash::{BuildHasher, Hasher};

		let state = RandomState::new();
		let mut hasher = state.build_hasher();
		hasher.write_usize(std::ptr::null::<()>() as usize);
		hasher.finish() as u32
	}

	pub fn mur_parse_multipart(body: &[u8], content_type: &str) -> Result<MurMultipart, MurError> {
		let boundary = Self::parse_boundary(content_type)?;
		Self::parse_multipart_body(
			&Bytes::from(body.to_vec()),
			&boundary,
			&MurMultipartConfig::default(),
		)
	}

	pub fn mur_parse_multipart_with_config(
		body: &[u8],
		content_type: &str,
		config: &MurMultipartConfig,
	) -> Result<MurMultipart, MurError> {
		let boundary = Self::parse_boundary(content_type)?;
		Self::parse_multipart_body(&Bytes::from(body.to_vec()), &boundary, config)
	}

	pub fn mur_is_multipart_content_type(content_type: &str) -> bool {
		content_type.to_lowercase().contains("multipart/form-data")
	}

	pub fn mur_extract_boundary(content_type: &str) -> Option<String> {
		Self::parse_boundary(content_type).ok()
	}
}
