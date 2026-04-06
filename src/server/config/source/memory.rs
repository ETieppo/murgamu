use super::MurConfigSource;
use crate::server::config::MurConfigResult;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MurMemorySource {
	values: HashMap<String, String>,
	priority: i32,
}

impl MurMemorySource {
	pub fn new() -> Self {
		Self {
			values: HashMap::new(),
			priority: 50,
		}
	}

	pub fn from_map(values: HashMap<String, String>) -> Self {
		Self {
			values,
			priority: 50,
		}
	}

	pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
		self.values.insert(key.into(), value.into());
		self
	}

	pub fn remove(&mut self, key: &str) -> Option<String> {
		self.values.remove(key)
	}

	pub fn clear(&mut self) {
		self.values.clear();
	}

	pub fn with_priority(mut self, priority: i32) -> Self {
		self.priority = priority;
		self
	}

	pub fn len(&self) -> usize {
		self.values.len()
	}

	pub fn is_empty(&self) -> bool {
		self.values.is_empty()
	}
}

impl MurConfigSource for MurMemorySource {
	fn load(&self) -> MurConfigResult<HashMap<String, String>> {
		Ok(self.values.clone())
	}

	fn name(&self) -> &str {
		"memory"
	}

	fn priority(&self) -> i32 {
		self.priority
	}
}
