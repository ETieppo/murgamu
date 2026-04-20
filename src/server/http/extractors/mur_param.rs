use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use std::ops::Deref;
use std::str::FromStr;

/// A typed wrapper for a single URL path parameter.
///
/// `Param<T>` extracts one named segment from the URL and parses it into `T`.
/// `T` must implement [`std::str::FromStr`].
///
/// # Usage in route handlers
///
/// ```rust,ignore
/// #[get("/users/:id")]
/// async fn get_user(&self, #[param] id: Param<u64>) -> MurRes {
///     println!("fetching user {}", *id);
///     mur_json!(serde_json::json!({ "id": *id }))
/// }
/// ```
///
/// For multiple path segments prefer [`MurPath<T>`](crate::MurPath).
#[derive(Debug, Clone)]
pub struct Param<T>(pub T);

impl<T> Param<T> {
	/// Wraps `value` in a `Param` extractor.
	pub fn new(value: T) -> Self {
		Self(value)
	}

	/// Consumes the wrapper and returns the inner value.
	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T> Deref for Param<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> AsRef<T> for Param<T> {
	fn as_ref(&self) -> &T {
		&self.0
	}
}

impl Param<String> {
	/// Extracts a named path parameter as a raw `String`.
	///
	/// Returns [`MurError::BadRequest`] if the parameter is not present in the
	/// matched route.
	pub fn extract(ctx: &MurRequestContext, name: &str) -> Result<Self, MurError> {
		ctx.path_params
			.get(name)
			.cloned()
			.map(Param)
			.ok_or_else(|| MurError::BadRequest(format!("Missing path parameter: {}", name)))
	}
}

impl<T: FromStr> Param<T>
where
	T::Err: std::fmt::Display,
{
	/// Extracts a named path parameter and parses it into `T`.
	///
	/// Returns [`MurError::BadRequest`] if the parameter is absent or if `FromStr`
	/// parsing fails.
	pub fn extract_parsed(ctx: &MurRequestContext, name: &str) -> Result<Self, MurError> {
		let value = ctx
			.path_params
			.get(name)
			.ok_or_else(|| MurError::BadRequest(format!("Missing path parameter: {}", name)))?;

		value
			.parse::<T>()
			.map(Param)
			.map_err(|e| MurError::BadRequest(format!("Invalid value for '{}': {}", name, e)))
	}
}
