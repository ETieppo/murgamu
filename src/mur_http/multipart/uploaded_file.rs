use super::MurMultipartUtils;
use crate::core::error::MurError;
use hyper::body::Bytes;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MurUploadedFile {
	pub filename: String,
	pub sanitized_filename: String,
	pub content_type: String,
	pub extension: Option<String>,
	pub data: Bytes,
	pub field_name: String,
}

impl MurUploadedFile {
	pub fn new(filename: String, content_type: String, data: Bytes, field_name: String) -> Self {
		let sanitized_filename = MurMultipartUtils::sanitize_filename(&filename);
		let extension = MurMultipartUtils::extract_extension(&filename);

		Self {
			filename,
			sanitized_filename,
			content_type,
			extension,
			data,
			field_name,
		}
	}

	pub fn filename(&self) -> &str {
		&self.filename
	}

	pub fn sanitized_filename(&self) -> &str {
		&self.sanitized_filename
	}

	pub fn content_type(&self) -> &str {
		&self.content_type
	}

	pub fn extension(&self) -> Option<&str> {
		self.extension.as_deref()
	}

	pub fn size(&self) -> usize {
		self.data.len()
	}

	pub fn data(&self) -> &Bytes {
		&self.data
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.data
	}

	pub fn into_bytes(self) -> Bytes {
		self.data
	}

	pub fn field_name(&self) -> &str {
		&self.field_name
	}

	pub fn has_extension(&self, ext: &str) -> bool {
		self.extension
			.as_ref()
			.map(|e| e.eq_ignore_ascii_case(ext))
			.unwrap_or(false)
	}

	pub fn has_extension_in(&self, extensions: &[&str]) -> bool {
		extensions.iter().any(|ext| self.has_extension(ext))
	}

	pub fn is_content_type(&self, content_type: &str) -> bool {
		self.content_type.eq_ignore_ascii_case(content_type)
	}

	pub fn is_image(&self) -> bool {
		self.content_type.starts_with("image/")
	}

	pub fn is_text(&self) -> bool {
		self.content_type.starts_with("text/")
	}

	pub fn is_pdf(&self) -> bool {
		self.content_type == "application/pdf"
	}

	pub fn as_text(&self) -> Result<&str, std::str::Utf8Error> {
		std::str::from_utf8(&self.data)
	}

	pub async fn save_to<P: AsRef<Path>>(&self, directory: P) -> Result<PathBuf, MurError> {
		self.save_to_with_name(directory, &self.sanitized_filename)
			.await
	}

	pub async fn save_to_with_name<P: AsRef<Path>>(
		&self,
		directory: P,
		filename: &str,
	) -> Result<PathBuf, MurError> {
		let path = directory.as_ref().join(filename);

		if let Some(parent) = path.parent() {
			tokio::fs::create_dir_all(parent)
				.await
				.map_err(|e| MurError::Internal(format!("Failed to create directory: {}", e)))?;
		}

		tokio::fs::write(&path, &self.data)
			.await
			.map_err(|e| MurError::Internal(format!("Failed to write file: {}", e)))?;

		Ok(path)
	}

	pub fn save_to_sync<P: AsRef<Path>>(&self, directory: P) -> Result<PathBuf, MurError> {
		let path = directory.as_ref().join(&self.sanitized_filename);

		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)
				.map_err(|e| MurError::Internal(format!("Failed to create directory: {}", e)))?;
		}

		let mut file = std::fs::File::create(&path)
			.map_err(|e| MurError::Internal(format!("Failed to create file: {}", e)))?;

		file.write_all(&self.data)
			.map_err(|e| MurError::Internal(format!("Failed to write file: {}", e)))?;

		Ok(path)
	}

	pub fn unique_filename(&self) -> String {
		let timestamp = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_millis())
			.unwrap_or(0);
		let random: u32 = MurMultipartUtils::rand_u32();

		match &self.extension {
			Some(ext) => format!("{}_{}.{}", timestamp, random, ext),
			None => format!("{}_{}", timestamp, random),
		}
	}
}
