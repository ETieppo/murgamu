use crate::mur_http::MurRequestContext;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct MurQueryParam<T> {
	pub value: Option<T>,
	_marker: PhantomData<T>,
}

impl<T> MurQueryParam<T> {
	pub fn new(value: Option<T>) -> Self {
		Self {
			value,
			_marker: PhantomData,
		}
	}

	pub fn is_present(&self) -> bool {
		self.value.is_some()
	}

	pub fn unwrap_or(self, default: T) -> T {
		self.value.unwrap_or(default)
	}

	pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
		self.value.unwrap_or_else(f)
	}
}

impl<T: std::str::FromStr> MurQueryParam<T> {
	pub fn extract(ctx: &MurRequestContext, name: &str) -> Self {
		let value = ctx.query_param(name).and_then(|s| s.parse().ok());
		Self {
			value,
			_marker: PhantomData,
		}
	}
}
