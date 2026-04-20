use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use std::ops::Deref;

/// A wrapper for plain-text request bodies.
///
/// `MurText` reads the request body as a UTF-8 string. Use it when the client
/// sends `text/plain` or any other non-JSON textual content type.
///
/// # Usage in route handlers
///
/// ```rust,ignore
/// #[post("/echo")]
/// async fn echo(&self, body: MurText) -> MurRes {
///     mur_json!(serde_json::json!({ "received": body.as_str() }))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MurText(pub String);

impl MurText {
	/// Consumes the wrapper and returns the inner `String`.
	pub fn into_inner(self) -> String {
		self.0
	}

	/// Wraps `text` in a `MurText` extractor.
	pub fn new(text: impl Into<String>) -> Self {
		Self(text.into())
	}

	/// Extracts and UTF-8-decodes the request body.
	///
	/// Returns [`MurError::BadRequest`] if the body is missing or contains
	/// invalid UTF-8 bytes.
	pub fn extract(ctx: &MurRequestContext) -> Result<Self, MurError> {
		ctx.body_string().map(MurText)
	}

	/// Extracts the body as text, returning an empty string when no body is present.
	pub fn extract_or_empty(ctx: &MurRequestContext) -> Self {
		MurText(ctx.body_string().unwrap_or_default())
	}

	/// Returns the number of bytes in the string.
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Returns `true` if the string is empty.
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Returns a string slice of the inner value.
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
