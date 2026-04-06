use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct MurHeader(pub String);

impl MurHeader {
	pub fn new(value: impl Into<String>) -> Self {
		Self(value.into())
	}

	pub fn extract(ctx: &MurRequestContext, name: &str) -> Result<Self, MurError> {
		ctx.header(name)
			.map(|s| MurHeader(s.to_string()))
			.ok_or_else(|| MurError::BadRequest(format!("Missing header: {}", name)))
	}

	pub fn extract_optional(ctx: &MurRequestContext, name: &str) -> Option<Self> {
		ctx.header(name).map(|s| MurHeader(s.to_string()))
	}

	pub fn into_inner(self) -> String {
		self.0
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl Deref for MurHeader {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl AsRef<str> for MurHeader {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl From<String> for MurHeader {
	fn from(s: String) -> Self {
		Self(s)
	}
}

impl From<&str> for MurHeader {
	fn from(s: &str) -> Self {
		Self(s.to_string())
	}
}
