use super::ExpectedCallCount;
use super::MockExpectation;
use super::MockExpectationBuilder;
use super::MockService;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct MockServiceBuilder<T> {
	pub name: String,
	pub return_values: HashMap<String, serde_json::Value>,
	pub expectations: Vec<MockExpectation>,
	_marker: PhantomData<T>,
}

impl<T> MockServiceBuilder<T> {
	pub fn new() -> Self {
		Self {
			name: std::any::type_name::<T>().to_string(),
			return_values: HashMap::new(),
			expectations: Vec::new(),
			_marker: PhantomData,
		}
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn returns_for<V: serde::Serialize>(mut self, method: &str, value: V) -> Self {
		if let Ok(json_value) = serde_json::to_value(value) {
			self.return_values.insert(method.to_string(), json_value);
		}
		self
	}

	pub fn expect_call(self, method: &str) -> MockExpectationBuilder<T> {
		MockExpectationBuilder {
			builder: self,
			method: method.to_string(),
			call_count: ExpectedCallCount::AtLeast(1),
		}
	}

	pub fn build(self) -> MockService {
		let mock = MockService::named(self.name);
		for (method, value) in self.return_values {
			mock.set_return_value(&method, value);
		}
		for exp in self.expectations {
			mock.expect(exp);
		}
		mock
	}
}

impl<T> Default for MockServiceBuilder<T> {
	fn default() -> Self {
		Self::new()
	}
}
