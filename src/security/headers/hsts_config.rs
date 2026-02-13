#[derive(Debug, Clone)]
pub struct HstsConfig {
	pub max_age: u64,
	pub include_subdomains: bool,
	pub preload: bool,
}

impl Default for HstsConfig {
	fn default() -> Self {
		Self {
			max_age: 15552000, // 180 days
			include_subdomains: false,
			preload: false,
		}
	}
}

impl HstsConfig {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn max_age(mut self, seconds: u64) -> Self {
		self.max_age = seconds;
		self
	}

	pub fn one_year(mut self) -> Self {
		self.max_age = 31536000;
		self
	}

	pub fn two_years(mut self) -> Self {
		self.max_age = 63072000;
		self
	}

	pub fn include_subdomains(mut self) -> Self {
		self.include_subdomains = true;
		self
	}

	pub fn preload(mut self) -> Self {
		self.preload = true;
		self
	}

	pub fn to_header_value(&self) -> String {
		let mut value = format!("max-age={}", self.max_age);
		if self.include_subdomains {
			value.push_str("; includeSubDomains");
		}
		if self.preload {
			value.push_str("; preload");
		}
		value
	}
}

impl std::fmt::Display for HstsConfig {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.to_header_value())
	}
}
