use crate::core::error::MurError;
use crate::mur_http::request::MurRequestContext;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct MurText(pub String);

impl MurText {
	pub fn into_inner(self) -> String {
		self.0
	}

	pub fn new(text: impl Into<String>) -> Self {
		Self(text.into())
	}

	pub fn extract(ctx: &MurRequestContext) -> Result<Self, MurError> {
		ctx.body_string().map(MurText)
	}

	pub fn extract_or_empty(ctx: &MurRequestContext) -> Self {
		MurText(ctx.body_string().unwrap_or_default())
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl Deref for MurText {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl AsRef<str> for MurText {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl From<String> for MurText {
	fn from(s: String) -> Self {
		Self(s)
	}
}

impl From<&str> for MurText {
	fn from(s: &str) -> Self {
		Self(s.to_string())
	}
}
