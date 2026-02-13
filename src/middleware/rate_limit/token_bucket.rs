use super::RateLimitStore;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct TokenBucketStore {
	data: RwLock<HashMap<String, TokenBucketEntry>>,
}

#[derive(Debug, Clone)]
struct TokenBucketEntry {
	pub tokens: f64,
	pub last_refill: Instant,
}

impl TokenBucketStore {
	pub fn new() -> Self {
		Self {
			data: RwLock::new(HashMap::new()),
		}
	}
}

impl Default for TokenBucketStore {
	fn default() -> Self {
		Self::new()
	}
}

impl RateLimitStore for TokenBucketStore {
	fn check_and_update(&self, key: &str, max_requests: u64, window: Duration) -> (bool, u64, u64) {
		let now = Instant::now();
		let mut data = self.data.write().unwrap();
		let refill_rate = max_requests as f64 / window.as_secs_f64();
		let entry = data
			.entry(key.to_string())
			.or_insert_with(|| TokenBucketEntry {
				tokens: max_requests as f64,
				last_refill: now,
			});

		let elapsed = now.duration_since(entry.last_refill);
		let new_tokens = elapsed.as_secs_f64() * refill_rate;
		entry.tokens = (entry.tokens + new_tokens).min(max_requests as f64);
		entry.last_refill = now;

		let allowed = entry.tokens >= 1.0;
		if allowed {
			entry.tokens -= 1.0;
		}

		let remaining = entry.tokens as u64;
		let tokens_needed = max_requests as f64 - entry.tokens;
		let seconds_until_full = (tokens_needed / refill_rate).ceil() as u64;
		let reset_at = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_secs()
			+ seconds_until_full;

		(allowed, remaining, reset_at)
	}

	fn reset(&self, key: &str) {
		let mut data = self.data.write().unwrap();
		data.remove(key);
	}

	fn get_status(&self, key: &str, max_requests: u64, window: Duration) -> (u64, u64, u64) {
		let data = self.data.read().unwrap();
		let refill_rate = max_requests as f64 / window.as_secs_f64();

		match data.get(key) {
			Some(entry) => {
				let now = Instant::now();
				let elapsed = now.duration_since(entry.last_refill);
				let new_tokens = elapsed.as_secs_f64() * refill_rate;
				let current_tokens = (entry.tokens + new_tokens).min(max_requests as f64);
				let used = max_requests as f64 - current_tokens;
				let remaining = current_tokens as u64;
				let tokens_needed = max_requests as f64 - current_tokens;
				let seconds_until_full = (tokens_needed / refill_rate).ceil() as u64;
				let reset_at = SystemTime::now()
					.duration_since(UNIX_EPOCH)
					.unwrap()
					.as_secs() + seconds_until_full;

				(used as u64, remaining, reset_at)
			}
			None => {
				let reset_at = SystemTime::now()
					.duration_since(UNIX_EPOCH)
					.unwrap()
					.as_secs();
				(0, max_requests, reset_at)
			}
		}
	}
}
