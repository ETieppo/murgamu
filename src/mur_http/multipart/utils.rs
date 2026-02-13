use super::MurFormField;
use super::MurMultipart;
use super::MurMultipartConfig;
use super::MurUploadedFile;
use crate::core::error::MurError;
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
		let _end_delimiter = format!("--{}--", boundary);
		let body_str = String::from_utf8_lossy(body);
		let parts: Vec<&str> = body_str.split(&delimiter).collect();
		let mut file_count = 0;
		let mut field_count = 0;

		for part in parts.iter().skip(1) {
			let part = part.trim_start_matches("\r\n").trim_end_matches("\r\n");
			if part.is_empty() || part.starts_with("--") {
				continue;
			}

			field_count += 1;
			if field_count > config.max_fields {
				return Err(MurError::BadRequest(format!(
					"Too many fields (max: {})",
					config.max_fields
				)));
			}

			let field = Self::parse_multipart_part(part, config)?;

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
						|| !config
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

	pub fn parse_multipart_part(
		part: &str,
		config: &MurMultipartConfig,
	) -> Result<MurFormField, MurError> {
		let (headers_str, body) = match part.find("\r\n\r\n") {
			Some(pos) => (&part[..pos], &part[pos + 4..]),
			None => match part.find("\n\n") {
				Some(pos) => (&part[..pos], &part[pos + 2..]),
				None => return Err(MurError::BadRequest("Invalid multipart part format".into())),
			},
		};
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
			let data = Bytes::from(body.as_bytes().to_vec());
			Ok(MurFormField::File(MurUploadedFile::new(
				filename,
				content_type,
				data,
				name,
			)))
		} else {
			Ok(MurFormField::Text {
				name,
				value: body.to_string(),
			})
		}
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
