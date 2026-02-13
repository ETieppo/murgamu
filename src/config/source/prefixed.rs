use super::MurConfigSource;
use crate::config::MurConfigResult;
use std::collections::HashMap;

pub struct MurPrefixedSource<S: MurConfigSource> {
	inner: S,
	prefix: String,
}

impl<S: MurConfigSource> MurPrefixedSource<S> {
	pub fn new(source: S, prefix: &str) -> Self {
		Self {
			inner: source,
			prefix: prefix.to_string(),
		}
	}
}

impl<S: MurConfigSource> MurConfigSource for MurPrefixedSource<S> {
	fn load(&self) -> MurConfigResult<HashMap<String, String>> {
		let values = self.inner.load()?;
		Ok(values
			.into_iter()
			.map(|(k, v)| (format!("{}{}", self.prefix, k), v))
			.collect())
	}

	fn name(&self) -> &str {
		self.inner.name()
	}

	fn priority(&self) -> i32 {
		self.inner.priority()
	}

	fn is_available(&self) -> bool {
		self.inner.is_available()
	}
}
