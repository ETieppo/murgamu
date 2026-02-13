use super::MurFormField;
use super::MurMultipartConfig;
use super::MurMultipartUtils;
use super::MurUploadedFile;
use crate::core::error::MurError;
use crate::mur_http::request::MurRequestContext;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MurMultipart {
	pub fields: Vec<MurFormField>,
	pub text_fields: HashMap<String, Vec<String>>,
	pub file_fields: HashMap<String, Vec<MurUploadedFile>>,
	pub total_file_size: usize,
}

impl MurMultipart {
	pub fn empty() -> Self {
		Self {
			fields: Vec::new(),
			text_fields: HashMap::new(),
			file_fields: HashMap::new(),
			total_file_size: 0,
		}
	}

	pub async fn parse(ctx: &MurRequestContext) -> Result<Self, MurError> {
		Self::parse_with_config(ctx, &MurMultipartConfig::default()).await
	}

	pub async fn parse_with_config(
		ctx: &MurRequestContext,
		config: &MurMultipartConfig,
	) -> Result<Self, MurError> {
		let content_type = ctx
			.header("Content-Type")
			.ok_or_else(|| MurError::BadRequest("Missing Content-Type header".into()))?;
		let boundary = MurMultipartUtils::parse_boundary(content_type)?;
		let body = ctx
			.body
			.as_ref()
			.ok_or_else(|| MurError::BadRequest("Missing request body".into()))?;

		if body.len() > config.max_body_size {
			return Err(MurError::BadRequest(format!(
				"Request body exceeds maximum size of {} bytes",
				config.max_body_size
			)));
		}

		MurMultipartUtils::parse_multipart_body(body, &boundary, config)
	}

	pub fn fields(&self) -> &[MurFormField] {
		&self.fields
	}

	pub fn fields_count(&self) -> usize {
		self.fields.len()
	}

	pub fn text_fields_count(&self) -> usize {
		self.text_fields.values().map(|v| v.len()).sum()
	}

	pub fn files_count(&self) -> usize {
		self.file_fields.values().map(|v| v.len()).sum()
	}

	pub fn total_file_size(&self) -> usize {
		self.total_file_size
	}

	pub fn has_files(&self) -> bool {
		!self.file_fields.is_empty()
	}

	pub fn has(&self, name: &str) -> bool {
		self.text_fields.contains_key(name) || self.file_fields.contains_key(name)
	}

	pub fn text(&self, name: &str) -> Option<&str> {
		self.text_fields
			.get(name)
			.and_then(|v| v.first())
			.map(|s| s.as_str())
	}

	pub fn text_all(&self, name: &str) -> Vec<&str> {
		self.text_fields
			.get(name)
			.map(|v| v.iter().map(|s| s.as_str()).collect())
			.unwrap_or_default()
	}

	pub fn text_or<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
		self.text(name).unwrap_or(default)
	}

	pub fn text_as<T: std::str::FromStr>(&self, name: &str) -> Option<T> {
		self.text(name).and_then(|s| s.parse().ok())
	}

	pub fn file(&self, name: &str) -> Option<&MurUploadedFile> {
		self.file_fields.get(name).and_then(|v| v.first())
	}

	pub fn files(&self, name: &str) -> Vec<&MurUploadedFile> {
		self.file_fields
			.get(name)
			.map(|v| v.iter().collect())
			.unwrap_or_default()
	}

	pub fn all_files(&self) -> Vec<&MurUploadedFile> {
		self.file_fields.values().flatten().collect()
	}

	pub fn take_file(&mut self, name: &str) -> Option<MurUploadedFile> {
		self.file_fields.get_mut(name).and_then(|v| {
			if v.is_empty() {
				None
			} else {
				Some(v.remove(0))
			}
		})
	}

	pub fn take_files(&mut self, name: &str) -> Vec<MurUploadedFile> {
		self.file_fields.remove(name).unwrap_or_default()
	}

	pub fn take_all_files(&mut self) -> Vec<MurUploadedFile> {
		let mut files = Vec::new();
		for (_, field_files) in self.file_fields.drain() {
			files.extend(field_files);
		}
		files
	}

	pub fn field_names(&self) -> Vec<&str> {
		let mut names: Vec<&str> = self
			.text_fields
			.keys()
			.chain(self.file_fields.keys())
			.map(|s| s.as_str())
			.collect();
		names.sort();
		names.dedup();
		names
	}

	pub fn to_text_map(&self) -> HashMap<String, String> {
		self.text_fields
			.iter()
			.filter_map(|(k, v)| v.first().map(|val| (k.clone(), val.clone())))
			.collect()
	}
}
