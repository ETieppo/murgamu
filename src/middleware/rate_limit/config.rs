use super::RateLimitKey;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitAlgorithm {
	FixedWindow,
	SlidingWindow,
	TokenBucket,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
	pub max_requests: u64,
	pub window: Duration,
	pub key_extractor: RateLimitKey,
	pub algorithm: RateLimitAlgorithm,
	pub message: Option<String>,
	pub include_headers: bool,
	pub skip_paths: Vec<String>,
	pub skip_on_missing_key: bool,
	pub status_code: u16,
}

impl Default for RateLimitConfig {
	fn default() -> Self {
		Self {
			max_requests: 100,
			window: Duration::from_secs(60),
			key_extractor: RateLimitKey::Ip,
			algorithm: RateLimitAlgorithm::FixedWindow,
			message: None,
			include_headers: true,
			skip_paths: Vec::new(),
			skip_on_missing_key: false,
			status_code: 429,
		}
	}
}
