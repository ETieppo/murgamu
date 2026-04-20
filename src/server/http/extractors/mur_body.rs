use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use hyper::body::Bytes;
use std::ops::Deref;

/// A wrapper for the raw request body as a byte buffer.
///
/// `MurBody` provides access to the unprocessed body bytes when you need
/// full control over parsing (binary protocols, custom formats, streaming, etc.).
///
/// # Usage in route handlers
///
/// ```rust,ignore
/// #[post("/upload")]
/// async fn upload(&self, body: MurBody) -> MurRes {
///     println!("received {} bytes", body.len());
///     mur_json!(serde_json::json!({ "size": body.len() }))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MurBody(pub Bytes);

impl MurBody {
	/// Wraps raw `bytes` in a `MurBody` extractor.
	pub fn new(bytes: Bytes) -> Self {
		Self(bytes)
	}

	/// Extracts the body from the request context.
	///
	/// Returns [`MurError::BadRequest`] when the request has no body.
	pub fn extract(ctx: &MurRequestContext) -> Result<Self, MurError> {
		ctx.body
			.clone()
			.map(MurBody)
			.ok_or_else(|| MurError::BadRequest("Missing request body".to_string()))
	}

	/// Extracts the body, returning an empty buffer when no body is present.
	pub fn extract_or_empty(ctx: &MurRequestContext) -> Self {
		MurBody(ctx.body.clone().unwrap_or_default())
	}

	/// Consumes the wrapper and returns the underlying [`Bytes`].
	pub fn into_inner(self) -> Bytes {
		self.0
	}

	/// Returns the number of bytes in the body.
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Returns `true` if the body contains no bytes.
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Copies the body contents into a `Vec<u8>`.
	pub fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec()
	}
}

impl Deref for MurBody {
	type Target = Bytes;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl AsRef<[u8]> for MurBody {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl From<Bytes> for MurBody {
	fn from(bytes: Bytes) -> Self {
		Self(bytes)
	}
}

impl From<Vec<u8>> for MurBody {
	fn from(vec: Vec<u8>) -> Self {
		Self(Bytes::from(vec))
	}
}
