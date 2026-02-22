use std::fmt;

#[derive(Debug)]
pub struct AssertionError {
	pub message: String,
	pub expected: String,
	pub actual: String,
}

impl AssertionError {
	pub fn new(
		message: impl Into<String>,
		expected: impl Into<String>,
		actual: impl Into<String>,
	) -> Self {
		Self {
			message: message.into(),
			expected: expected.into(),
			actual: actual.into(),
		}
	}

	pub fn missing(what: impl Into<String>) -> Self {
		Self {
			message: format!("{} is missing", what.into()),
			expected: "present".to_string(),
			actual: "missing".to_string(),
		}
	}
}

impl fmt::Display for AssertionError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"Assertion failed: {}\n  Expected: {}\n  Actual: {}",
			self.message, self.expected, self.actual
		)
	}
}

impl std::error::Error for AssertionError {}
