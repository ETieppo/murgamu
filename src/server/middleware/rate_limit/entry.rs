use std::time::Instant;

#[derive(Debug, Clone)]
pub struct RateLimitEntry {
	pub count: u64,
	pub window_start: Instant,
	pub prev_count: u64,
	pub tokens: f64,
	pub last_refill: Instant,
}

impl RateLimitEntry {
	pub fn new() -> Self {
		let now = Instant::now();
		Self {
			count: 0,
			window_start: now,
			prev_count: 0,
			tokens: 0.0,
			last_refill: now,
		}
	}
}

impl Default for RateLimitEntry {
	fn default() -> Self {
		Self::new()
	}
}
