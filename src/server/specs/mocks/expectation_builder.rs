use super::MockServiceBuilder;

#[derive(Debug, Clone)]
pub struct MockExpectation {
	pub method: String,
	pub call_count: ExpectedCallCount,
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedCallCount {
	Exactly(usize),
	AtLeast(usize),
	AtMost(usize),
	Between(usize, usize),
	Never,
}

pub struct MockExpectationBuilder<T> {
	pub builder: MockServiceBuilder<T>,
	pub method: String,
	pub call_count: ExpectedCallCount,
}

impl<T> MockExpectationBuilder<T> {
	pub fn times(mut self, n: usize) -> Self {
		self.call_count = ExpectedCallCount::Exactly(n);
		self
	}

	pub fn at_least(mut self, n: usize) -> Self {
		self.call_count = ExpectedCallCount::AtLeast(n);
		self
	}

	pub fn at_most(mut self, n: usize) -> Self {
		self.call_count = ExpectedCallCount::AtMost(n);
		self
	}

	pub fn between(mut self, min: usize, max: usize) -> Self {
		self.call_count = ExpectedCallCount::Between(min, max);
		self
	}

	pub fn never(mut self) -> Self {
		self.call_count = ExpectedCallCount::Never;
		self
	}

	pub fn returns<V: serde::Serialize>(mut self, value: V) -> MockServiceBuilder<T> {
		if let Ok(json_value) = serde_json::to_value(value) {
			self.builder
				.return_values
				.insert(self.method.clone(), json_value);
		}

		self.builder.expectations.push(MockExpectation {
			method: self.method,
			call_count: self.call_count,
		});

		self.builder
	}

	pub fn and(self) -> MockServiceBuilder<T> {
		let mut builder = self.builder;
		builder.expectations.push(MockExpectation {
			method: self.method,
			call_count: self.call_count,
		});
		builder
	}
}
