use crate::core::error::MurError;
use crate::mur_http::request::MurRequestContext;
use serde::de::DeserializeOwned;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct MurJson<T>(pub T);

impl<T> MurJson<T> {
	pub fn new(value: T) -> Self {
		Self(value)
	}

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
	pub fn extract(ctx: &MurRequestContext) -> Result<Self, MurError> {
		ctx.json().map(MurJson)
	}
}

impl<T> AsRef<T> for MurJson<T> {
	fn as_ref(&self) -> &T {
		&self.0
	}
}
