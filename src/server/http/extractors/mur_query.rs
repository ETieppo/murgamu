use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use serde::de::DeserializeOwned;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct MurQuery<T>(pub T);

impl<T> MurQuery<T> {
	pub fn new(value: T) -> Self {
		Self(value)
	}

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
