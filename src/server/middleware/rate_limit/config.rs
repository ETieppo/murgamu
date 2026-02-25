use super::MurThrottlerKey;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MurThrottlerAlgorithm {
	FixedWindow,
	SlidingWindow,
	TokenBucket,
}

#[derive(Debug, Clone)]
pub struct MurThrottlerConfig {
	pub max_requests: u64,
	pub window: Duration,
	pub key_extractor: MurThrottlerKey,
	pub algorithm: MurThrottlerAlgorithm,
	pub message: Option<String>,
	pub include_headers: bool,
	pub skip_paths: Vec<String>,
	pub skip_on_missing_key: bool,
	pub status_code: u16,
}

impl Default for MurThrottlerConfig {
	fn default() -> Self {
		Self {
			max_requests: 100,
			window: Duration::from_secs(60),
			key_extractor: MurThrottlerKey::Ip,
			algorithm: MurThrottlerAlgorithm::FixedWindow,
			message: None,
			include_headers: true,
			skip_paths: Vec::new(),
			skip_on_missing_key: false,
			status_code: 429,
		}
	}
}
