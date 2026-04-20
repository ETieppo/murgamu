use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use serde::de::DeserializeOwned;
use std::ops::Deref;

/// A typed wrapper for URL query-string parameters.
///
/// `MurQuery<T>` deserializes the request query string into `T` using
/// `serde_urlencoded`. `T` must implement [`serde::Deserialize`].
///
/// # Usage in route handlers
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct SearchParams {
///     q: String,
///     page: Option<u32>,
/// }
///
/// #[get("/search")]
/// async fn search(&self, query: MurQuery<SearchParams>) -> MurRes {
///     println!("searching: {}", query.q);
///     mur_json!(serde_json::json!({ "query": query.q }))
/// }
/// ```
///
/// The inner value is accessible via [`Deref`] (dot syntax), `.0`, or
/// [`MurQuery::into_inner`].
#[derive(Debug, Clone)]
pub struct MurQuery<T>(pub T);

impl<T> MurQuery<T> {
	/// Wraps `value` in a `MurQuery` extractor.
	pub fn new(value: T) -> Self {
		Self(value)
	}

	/// Consumes the wrapper and returns the inner value.
	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T> Deref for MurQuery<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: DeserializeOwned + Default> MurQuery<T> {
	/// Extracts and deserializes the query string.
	///
	/// Returns `T::default()` when the query string is absent or empty.
	/// Returns [`MurError::BadRequest`] if parsing fails.
	pub fn extract(ctx: &MurRequestContext) -> Result<Self, MurError> {
		let query_string = ctx.parts.uri.query().unwrap_or("");
		if query_string.is_empty() {
			return Ok(MurQuery(T::default()));
		}

		serde_urlencoded::from_str(query_string)
			.map(MurQuery)
			.map_err(|e| MurError::BadRequest(format!("Failed to parse query params: {}", e)))
	}
}

impl<T: DeserializeOwned> MurQuery<T> {
	/// Extracts the query string, returning an error if it is absent.
	///
	/// Use this variant when the query string is mandatory for the handler.
	pub fn extract_required(ctx: &MurRequestContext) -> Result<Self, MurError> {
		let query_string = ctx
			.parts
			.uri
			.query()
			.ok_or_else(|| MurError::BadRequest("Query string required".to_string()))?;

		serde_urlencoded::from_str(query_string)
			.map(MurQuery)
			.map_err(|e| MurError::BadRequest(format!("Failed to parse query params: {}", e)))
	}
}

impl<T> AsRef<T> for MurQuery<T> {
	fn as_ref(&self) -> &T {
		&self.0
	}
}
