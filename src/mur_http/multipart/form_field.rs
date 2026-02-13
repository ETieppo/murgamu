use super::MurUploadedFile;

#[derive(Debug, Clone)]
pub enum MurFormField {
	Text { name: String, value: String },
	File(MurUploadedFile),
}

impl MurFormField {
	pub fn name(&self) -> &str {
		match self {
			MurFormField::Text { name, .. } => name,
			MurFormField::File(file) => &file.field_name,
		}
	}

	pub fn is_text(&self) -> bool {
		matches!(self, MurFormField::Text { .. })
	}

	pub fn is_file(&self) -> bool {
		matches!(self, MurFormField::File(_))
	}

	pub fn as_text(&self) -> Option<&str> {
		match self {
			MurFormField::Text { value, .. } => Some(value),
			MurFormField::File(_) => None,
		}
	}

	pub fn as_file(&self) -> Option<&MurUploadedFile> {
		match self {
			MurFormField::Text { .. } => None,
			MurFormField::File(file) => Some(file),
		}
	}

	pub fn into_file(self) -> Option<MurUploadedFile> {
		match self {
			MurFormField::Text { .. } => None,
			MurFormField::File(file) => Some(file),
		}
	}
}
