use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use serde::de::DeserializeOwned;
use std::ops::Deref;

/// A typed wrapper for JSON request bodies.
///
/// `MurJson<T>` deserializes the raw request body as JSON into `T` automatically
/// when used as a handler parameter. `T` must implement [`serde::Deserialize`].
///
/// # Usage in route handlers
///
/// ```rust,ignore
/// #[post("/users")]
/// async fn create_user(&self, body: MurJson<CreateUserDto>) -> MurRes {
///     println!("name: {}", body.name);
///     mur_json!(body.0)
/// }
/// ```
///
/// The inner value is accessible via [`Deref`] (dot syntax), `.0`, or
/// [`MurJson::into_inner`].
#[derive(Debug, Clone)]
pub struct MurJson<T>(pub T);

impl<T> MurJson<T> {
	/// Wraps `value` in a `MurJson` extractor.
	pub fn new(value: T) -> Self {
		Self(value)
	}

	/// Consumes the wrapper and returns the inner value.
	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T> Deref for MurJson<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: DeserializeOwned> MurJson<T> {
	/// Extracts and deserializes the JSON body from the request context.
	///
	/// Returns [`MurError::BadRequest`] if the body is missing or not valid JSON.
	pub fn extract(ctx: &MurRequestContext) -> Result<Self, MurError> {
		ctx.json().map(MurJson)
	}
}

impl<T> AsRef<T> for MurJson<T> {
	fn as_ref(&self) -> &T {
		&self.0
	}
}
