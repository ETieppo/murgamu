use super::InMemoryStore;
use super::RateLimitStore;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct SlidingWindowStore {
	pub inner: InMemoryStore,
}

impl SlidingWindowStore {
	pub fn new() -> Self {
		Self {
			inner: InMemoryStore::new(),
		}
	}
}

impl Default for SlidingWindowStore {
	fn default() -> Self {
		Self::new()
	}
}

impl RateLimitStore for SlidingWindowStore {
	fn check_and_update(&self, key: &str, max_requests: u64, window: Duration) -> (bool, u64, u64) {
		self.inner.cleanup(window);

		let now = Instant::now();
		let mut data = self.inner.data.write().unwrap();
		let entry = data.entry(key.to_string()).or_default();
		let elapsed = now.duration_since(entry.window_start);

		if elapsed >= window {
			entry.prev_count = entry.count;
			entry.count = 0;
			entry.window_start = now;
		}

		let weight = 1.0 - (elapsed.as_secs_f64() / window.as_secs_f64());
		let weighted_count = entry.count as f64 + (entry.prev_count as f64 * weight.max(0.0));
		entry.count += 1;

		let allowed = weighted_count < max_requests as f64;
		let remaining = if allowed {
			(max_requests as f64 - weighted_count - 1.0).max(0.0) as u64
		} else {
			0
		};

		let reset_at = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_secs()
			+ (window.as_secs() - elapsed.as_secs().min(window.as_secs()));

		(allowed, remaining, reset_at)
	}

	fn reset(&self, key: &str) {
		self.inner.reset(key);
	}

	fn get_status(&self, key: &str, max_requests: u64, window: Duration) -> (u64, u64, u64) {
		let data = self.inner.data.read().unwrap();

		match data.get(key) {
			Some(entry) => {
				let now = Instant::now();
				let elapsed = now.duration_since(entry.window_start);
				let weight = 1.0 - (elapsed.as_secs_f64() / window.as_secs_f64());
				let weighted_count =
					entry.count as f64 + (entry.prev_count as f64 * weight.max(0.0));
				let remaining = (max_requests as f64 - weighted_count).max(0.0) as u64;
				let reset_at = SystemTime::now()
					.duration_since(UNIX_EPOCH)
					.unwrap()
					.as_secs() + (window.as_secs()
					- elapsed.as_secs().min(window.as_secs()));

				(weighted_count as u64, remaining, reset_at)
			}
			None => {
				let reset_at = SystemTime::now()
					.duration_since(UNIX_EPOCH)
					.unwrap()
					.as_secs() + window.as_secs();
				(0, max_requests, reset_at)
			}
		}
	}
}
