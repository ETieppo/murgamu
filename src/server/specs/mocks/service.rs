use super::ExpectedCallCount;
use super::MockExpectation;
use crate::server::service::MurService;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

pub struct MockService {
	#[allow(dead_code)]
	name: String,
	return_values: RwLock<HashMap<String, serde_json::Value>>,
	call_counts: RwLock<HashMap<String, usize>>,
	expectations: RwLock<Vec<MockExpectation>>,
	record_calls: AtomicBool,
}

impl MockService {
	pub fn new() -> Self {
		Self {
			name: "MockService".to_string(),
			return_values: RwLock::new(HashMap::new()),
			call_counts: RwLock::new(HashMap::new()),
			expectations: RwLock::new(Vec::new()),
			record_calls: AtomicBool::new(true),
		}
	}

	pub fn named(name: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			..Self::new()
		}
	}

	pub fn set_return_value(&self, method: &str, value: serde_json::Value) {
		self.return_values
			.write()
			.unwrap()
			.insert(method.to_string(), value);
	}

	pub fn get_return_value(&self, method: &str) -> Option<serde_json::Value> {
		if self.record_calls.load(Ordering::SeqCst) {
			let mut counts = self.call_counts.write().unwrap();
			*counts.entry(method.to_string()).or_insert(0) += 1;
		}
		self.return_values.read().unwrap().get(method).cloned()
	}

	pub fn call<T: serde::de::DeserializeOwned>(&self, method: &str) -> Option<T> {
		self.get_return_value(method)
			.and_then(|v| serde_json::from_value(v).ok())
	}

	pub fn call_count(&self, method: &str) -> usize {
		self.call_counts
			.read()
			.unwrap()
			.get(method)
			.copied()
			.unwrap_or(0)
	}

	pub fn was_called(&self, method: &str) -> bool {
		self.call_count(method) > 0
	}

	pub fn was_called_times(&self, method: &str, times: usize) -> bool {
		self.call_count(method) == times
	}

	pub fn reset_calls(&self) {
		self.call_counts.write().unwrap().clear();
	}

	pub fn reset_returns(&self) {
		self.return_values.write().unwrap().clear();
	}

	pub fn reset(&self) {
		self.reset_calls();
		self.reset_returns();
	}

	pub fn expect(&self, expectation: MockExpectation) {
		self.expectations.write().unwrap().push(expectation);
	}

	pub fn verify(&self) -> Result<(), String> {
		let expectations = self.expectations.read().unwrap();
		let counts = self.call_counts.read().unwrap();

		for exp in expectations.iter() {
			let actual_count = counts.get(&exp.method).copied().unwrap_or(0);

			match exp.call_count {
				ExpectedCallCount::Exactly(n) => {
					if actual_count != n {
						return Err(format!(
							"Expected '{}' to be called {} times, but was called {} times",
							exp.method, n, actual_count
						));
					}
				}
				ExpectedCallCount::AtLeast(n) => {
					if actual_count < n {
						return Err(format!(
							"Expected '{}' to be called at least {} times, but was called {} times",
							exp.method, n, actual_count
						));
					}
				}
				ExpectedCallCount::AtMost(n) => {
					if actual_count > n {
						return Err(format!(
							"Expected '{}' to be called at most {} times, but was called {} times",
							exp.method, n, actual_count
						));
					}
				}
				ExpectedCallCount::Between(min, max) => {
					if actual_count < min || actual_count > max {
						return Err(format!(
                            "Expected '{}' to be called between {} and {} times, but was called {} times",
                            exp.method, min, max, actual_count
                        ));
					}
				}
				ExpectedCallCount::Never => {
					if actual_count != 0 {
						return Err(format!(
							"Expected '{}' to never be called, but was called {} times",
							exp.method, actual_count
						));
					}
				}
			}
		}

		Ok(())
	}
}

impl Default for MockService {
	fn default() -> Self {
		Self::new()
	}
}

impl MurService for MockService {
	fn as_any(&self) -> &dyn Any {
		self
	}
}
