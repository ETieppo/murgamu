use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use std::ops::Deref;

/// A wrapper for a single HTTP request header value.
///
/// `MurHeader` extracts the value of a named header from the incoming request.
///
/// # Usage in route handlers
///
/// ```rust,ignore
/// #[get("/protected")]
/// async fn protected(&self, ctx: MurRequestContext) -> MurRes {
///     let auth = MurHeader::extract(&ctx, "Authorization")?;
///     println!("token: {}", auth.as_str());
///     mur_json!(serde_json::json!({ "ok": true }))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MurHeader(pub String);

impl MurHeader {
	/// Wraps `value` in a `MurHeader` extractor.
	pub fn new(value: impl Into<String>) -> Self {
		Self(value.into())
	}

	/// Extracts the named header, returning an error if it is absent.
	///
	/// Returns [`MurError::BadRequest`] when the header is not present in the request.
	pub fn extract(ctx: &MurRequestContext, name: &str) -> Result<Self, MurError> {
		ctx.header(name)
			.map(|s| MurHeader(s.to_string()))
			.ok_or_else(|| MurError::BadRequest(format!("Missing header: {}", name)))
	}

	/// Extracts the named header, returning `None` if it is absent.
	pub fn extract_optional(ctx: &MurRequestContext, name: &str) -> Option<Self> {
		ctx.header(name).map(|s| MurHeader(s.to_string()))
	}

	/// Consumes the wrapper and returns the inner `String`.
	pub fn into_inner(self) -> String {
		self.0
	}

	/// Returns a string slice of the header value.
	pub fn as_str(&self) -> &str {
		&self.0
	}

	/// Returns `true` if the header value is an empty string.
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
