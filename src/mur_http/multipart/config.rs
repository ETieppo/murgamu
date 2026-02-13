#[derive(Debug, Clone)]
pub struct MurMultipartConfig {
	pub max_body_size: usize,
	pub max_file_size: usize,
	pub max_fields: usize,
	pub max_files: usize,
	pub max_field_name_length: usize,
	pub allowed_extensions: Vec<String>,
	pub allowed_mime_types: Vec<String>,
}

impl Default for MurMultipartConfig {
	fn default() -> Self {
		Self {
			max_body_size: 50 * 1024 * 1024,
			max_file_size: 10 * 1024 * 1024,
			max_fields: 100,
			max_files: 20,
			max_field_name_length: 256,
			allowed_extensions: Vec::new(),
			allowed_mime_types: Vec::new(),
		}
	}
}

impl MurMultipartConfig {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn max_body_size(mut self, size: usize) -> Self {
		self.max_body_size = size;
		self
	}

	pub fn max_file_size(mut self, size: usize) -> Self {
		self.max_file_size = size;
		self
	}

	pub fn max_fields(mut self, count: usize) -> Self {
		self.max_fields = count;
		self
	}

	pub fn max_files(mut self, count: usize) -> Self {
		self.max_files = count;
		self
	}

	pub fn allowed_extensions<I, S>(mut self, extensions: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		self.allowed_extensions = extensions.into_iter().map(|s| s.into()).collect();
		self
	}

	pub fn allowed_mime_types<I, S>(mut self, types: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		self.allowed_mime_types = types.into_iter().map(|s| s.into()).collect();
		self
	}

	pub fn permissive() -> Self {
		Self {
			max_body_size: 500 * 1024 * 1024,
			max_file_size: 100 * 1024 * 1024,
			max_fields: 1000,
			max_files: 100,
			max_field_name_length: 1024,
			allowed_extensions: Vec::new(),
			allowed_mime_types: Vec::new(),
		}
	}

	pub fn strict() -> Self {
		Self {
			max_body_size: 5 * 1024 * 1024,
			max_file_size: 1024 * 1024,
			max_fields: 20,
			max_files: 5,
			max_field_name_length: 128,
			allowed_extensions: Vec::new(),
			allowed_mime_types: Vec::new(),
		}
	}

	pub fn images_only() -> Self {
		Self::default()
			.allowed_extensions(["jpg", "jpeg", "png", "gif", "webp", "svg"])
			.allowed_mime_types([
				"image/jpeg",
				"image/png",
				"image/gif",
				"image/webp",
				"image/svg+xml",
			])
	}

	pub fn documents_only() -> Self {
		Self::default()
			.allowed_extensions(["pdf", "doc", "docx", "xls", "xlsx", "txt", "csv"])
			.allowed_mime_types([
				"application/pdf",
				"application/msword",
				"application/vnd.openxmlformats-officedocument.wordprocessingml.document",
				"application/vnd.ms-excel",
				"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
				"text/plain",
				"text/csv",
			])
	}
}
