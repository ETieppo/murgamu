use crate::core::error::MurError;
use crate::mur_http::request::MurRequestContext;
use hyper::body::Bytes;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct MurBody(pub Bytes);

impl MurBody {
	pub fn new(bytes: Bytes) -> Self {
		Self(bytes)
	}

	pub fn extract(ctx: &MurRequestContext) -> Result<Self, MurError> {
		ctx.body
			.clone()
			.map(MurBody)
			.ok_or_else(|| MurError::BadRequest("Missing request body".to_string()))
	}

	pub fn extract_or_empty(ctx: &MurRequestContext) -> Self {
		MurBody(ctx.body.clone().unwrap_or_default())
	}

	pub fn into_inner(self) -> Bytes {
		self.0
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

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
