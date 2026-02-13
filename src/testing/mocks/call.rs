#[derive(Debug, Clone)]
pub struct MockCall {
	pub method: String,
	pub args: Option<serde_json::Value>,
	pub timestamp: std::time::Instant,
}

impl MockCall {
	pub fn new(method: impl Into<String>) -> Self {
		Self {
			method: method.into(),
			args: None,
			timestamp: std::time::Instant::now(),
		}
	}

	pub fn with_args<A: serde::Serialize>(method: impl Into<String>, args: A) -> Self {
		Self {
			method: method.into(),
			args: serde_json::to_value(args).ok(),
			timestamp: std::time::Instant::now(),
		}
	}
}