use super::MurConfigSource;
use crate::config::MurConfigResult;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MurEnvSource {
	pub prefix: Option<String>,
	pub strip_prefix: bool,
	pub lowercase_keys: bool,
}

impl MurEnvSource {
	pub fn new() -> Self {
		Self {
			prefix: None,
			strip_prefix: false,
			lowercase_keys: false,
		}
	}

	pub fn with_prefix(prefix: &str) -> Self {
		Self {
			prefix: Some(prefix.to_string()),
			strip_prefix: true,
			lowercase_keys: false,
		}
	}

	pub fn strip_prefix(mut self, strip: bool) -> Self {
		self.strip_prefix = strip;
		self
	}

	pub fn lowercase_keys(mut self, lowercase: bool) -> Self {
		self.lowercase_keys = lowercase;
		self
	}
}

impl Default for MurEnvSource {
	fn default() -> Self {
		Self::new()
	}
}

impl MurConfigSource for MurEnvSource {
	fn load(&self) -> MurConfigResult<HashMap<String, String>> {
		let mut values = HashMap::new();

		for (key, value) in std::env::vars() {
			let key = if let Some(prefix) = &self.prefix {
				if key.starts_with(prefix) {
					if self.strip_prefix {
						key[prefix.len()..].to_string()
					} else {
						key
					}
				} else {
					continue;
				}
			} else {
				key
			};

			let key = if self.lowercase_keys {
				key.to_lowercase()
			} else {
				key
			};

			values.insert(key, value);
		}

		Ok(values)
	}

	fn name(&self) -> &str {
		"environment"
	}

	fn priority(&self) -> i32 {
		100
	}
}
