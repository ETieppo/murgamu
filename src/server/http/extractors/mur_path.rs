use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use serde::de::DeserializeOwned;
use std::ops::Deref;

/// A typed wrapper for URL path parameters.
///
/// `MurPath<T>` deserializes all named path segments captured by the route
/// pattern into a struct `T`. `T` must implement [`serde::Deserialize`].
///
/// # Usage in route handlers
///
/// ```rust,ignore
/// #[derive(Deserialize)]
/// struct UserParams {
///     id: u64,
/// }
///
/// #[get("/users/:id")]
/// async fn get_user(&self, params: MurPath<UserParams>) -> MurRes {
///     println!("user id: {}", params.id);
///     mur_json!(serde_json::json!({ "id": params.id }))
/// }
/// ```
///
/// The inner value is accessible via [`Deref`] (dot syntax), `.0`, or
/// [`MurPath::into_inner`].
#[derive(Debug, Clone)]
pub struct MurPath<T>(pub T);

impl<T> MurPath<T> {
	/// Wraps `value` in a `MurPath` extractor.
	pub fn new(value: T) -> Self {
		Self(value)
	}

	/// Consumes the wrapper and returns the inner value.
	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T> Deref for MurPath<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl MurPath<String> {
	/// Extracts a single named path parameter as a `String`.
	///
	/// Returns [`MurError::BadRequest`] if the parameter is absent from the route match.
	pub fn extract(ctx: &MurRequestContext, name: &str) -> Result<Self, MurError> {
		ctx.path_param(name)
			.map(|s| MurPath(s.to_string()))
			.ok_or_else(|| MurError::BadRequest(format!("Missing path parameter: {}", name)))
	}
}

impl<T: DeserializeOwned> MurPath<T> {
	/// Deserializes all captured path parameters into `T` at once.
	///
	/// Useful when the route has multiple segments (e.g. `/orgs/:org/repos/:repo`).
	/// Returns [`MurError::BadRequest`] if any required field is missing or unparseable.
	pub fn extract_all(ctx: &MurRequestContext) -> Result<Self, MurError> {
		let json_value = serde_json::to_value(&ctx.path_params)
			.map_err(|e| MurError::Internal(format!("Failed to serialize path params: {}", e)))?;

		serde_json::from_value(json_value)
			.map(MurPath)
			.map_err(|e| MurError::BadRequest(format!("Failed to parse path params: {}", e)))
	}
}

impl<T> AsRef<T> for MurPath<T> {
	fn as_ref(&self) -> &T {
		&self.0
	}
}
