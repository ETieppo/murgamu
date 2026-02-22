use super::MurConfigSource;
use crate::server::config::MurConfigResult;
use std::collections::HashMap;

#[derive(Default)]
pub struct MurChainedSource {
	sources: Vec<Box<dyn MurConfigSource>>,
}

impl MurChainedSource {
	pub fn new() -> Self {
		Self {
			sources: Vec::new(),
		}
	}

	pub fn add_source<S: MurConfigSource + 'static>(mut self, source: S) -> Self {
		self.sources.push(Box::new(source));
		self
	}

	pub fn add_if<S: MurConfigSource + 'static>(self, condition: bool, source: S) -> Self {
		if condition {
			self.add_source(source)
		} else {
			self
		}
	}

	pub fn len(&self) -> usize {
		self.sources.len()
	}

	pub fn is_empty(&self) -> bool {
		self.sources.is_empty()
	}
}

impl MurConfigSource for MurChainedSource {
	fn load(&self) -> MurConfigResult<HashMap<String, String>> {
		let mut values = HashMap::new();
		let mut sorted_sources: Vec<_> = self.sources.iter().collect();
		sorted_sources.sort_by_key(|s| s.priority());

		for source in sorted_sources {
			if source.is_available() {
				match source.load() {
					Ok(source_values) => {
						values.extend(source_values);
					}
					Err(e) => {
						eprintln!(
							"Warning: Failed to load config from '{}': {}",
							source.name(),
							e
						);
					}
				}
			}
		}

		Ok(values)
	}

	fn name(&self) -> &str {
		"chained"
	}

	fn priority(&self) -> i32 {
		self.sources.iter().map(|s| s.priority()).max().unwrap_or(0)
	}
}
