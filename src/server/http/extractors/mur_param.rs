use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Param<T>(pub T);

impl<T> Param<T> {
	pub fn new(value: T) -> Self {
		Self(value)
	}

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
