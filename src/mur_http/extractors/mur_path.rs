use crate::core::error::MurError;
use crate::mur_http::request::MurRequestContext;
use serde::de::DeserializeOwned;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct MurPath<T>(pub T);

impl<T> MurPath<T> {
	pub fn new(value: T) -> Self {
		Self(value)
	}

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
	pub fn extract(ctx: &MurRequestContext, name: &str) -> Result<Self, MurError> {
		ctx.path_param(name)
			.map(|s| MurPath(s.to_string()))
			.ok_or_else(|| MurError::BadRequest(format!("Missing path parameter: {}", name)))
	}
}

impl<T: DeserializeOwned> MurPath<T> {
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
