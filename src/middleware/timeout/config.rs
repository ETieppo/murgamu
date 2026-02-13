use hyper::StatusCode;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TimeoutConfig {
	pub timeout: Duration,
	pub status_code: StatusCode,
	pub message: Option<String>,
	pub skip_paths: Vec<String>,
	pub skip_path_prefixes: Vec<String>,
	pub include_timeout_header: bool,
	pub timeout_header_name: String,
	pub log_timeouts: bool,
}

impl TimeoutConfig {
	pub fn new(timeout: Duration) -> Self {
		Self {
			timeout,
			status_code: StatusCode::REQUEST_TIMEOUT,
			message: None,
			skip_paths: Vec::new(),
			skip_path_prefixes: Vec::new(),
			include_timeout_header: false,
			timeout_header_name: "X-Timeout-Duration".to_string(),
			log_timeouts: true,
		}
	}

	pub fn from_secs(secs: u64) -> Self {
		Self::new(Duration::from_secs(secs))
	}

	pub fn from_millis(millis: u64) -> Self {
		Self::new(Duration::from_millis(millis))
	}

	pub fn status_code(mut self, status: StatusCode) -> Self {
		self.status_code = status;
		self
	}

	pub fn gateway_timeout(mut self) -> Self {
		self.status_code = StatusCode::GATEWAY_TIMEOUT;
		self
	}

	pub fn message(mut self, message: impl Into<String>) -> Self {
		self.message = Some(message.into());
		self
	}

	pub fn skip_paths(mut self, paths: Vec<impl Into<String>>) -> Self {
		self.skip_paths = paths.into_iter().map(|p| p.into()).collect();
		self
	}

	pub fn skip_path(mut self, path: impl Into<String>) -> Self {
		self.skip_paths.push(path.into());
		self
	}

	pub fn skip_path_prefixes(mut self, prefixes: Vec<impl Into<String>>) -> Self {
		self.skip_path_prefixes = prefixes.into_iter().map(|p| p.into()).collect();
		self
	}

	pub fn skip_path_prefix(mut self, prefix: impl Into<String>) -> Self {
		self.skip_path_prefixes.push(prefix.into());
		self
	}

	pub fn include_timeout_header(mut self, include: bool) -> Self {
		self.include_timeout_header = include;
		self
	}

	pub fn timeout_header_name(mut self, name: impl Into<String>) -> Self {
		self.timeout_header_name = name.into();
		self
	}

	pub fn log_timeouts(mut self, log: bool) -> Self {
		self.log_timeouts = log;
		self
	}
}

impl Default for TimeoutConfig {
	fn default() -> Self {
		Self::new(Duration::from_secs(30))
	}
}
