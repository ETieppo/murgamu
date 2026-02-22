use super::RateLimitEntry;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub trait RateLimitStore: Send + Sync + 'static {
	fn check_and_update(&self, key: &str, max_requests: u64, window: Duration) -> (bool, u64, u64);
	fn reset(&self, key: &str);
	fn get_status(&self, key: &str, max_requests: u64, window: Duration) -> (u64, u64, u64);
}

#[derive(Debug)]
pub struct InMemoryStore {
	pub data: RwLock<HashMap<String, RateLimitEntry>>,
	pub cleanup_interval: Duration,
	pub last_cleanup: RwLock<Instant>,
}

impl InMemoryStore {
	pub fn new() -> Self {
		Self {
			data: RwLock::new(HashMap::new()),
			cleanup_interval: Duration::from_secs(300),
			last_cleanup: RwLock::new(Instant::now()),
		}
	}

	pub fn with_cleanup_interval(interval: Duration) -> Self {
		Self {
			data: RwLock::new(HashMap::new()),
			cleanup_interval: interval,
			last_cleanup: RwLock::new(Instant::now()),
		}
	}

	pub fn cleanup(&self, max_age: Duration) {
		let mut last_cleanup = self.last_cleanup.write().unwrap();
		if last_cleanup.elapsed() < self.cleanup_interval {
			return;
		}

		let mut data = self.data.write().unwrap();
		let now = Instant::now();

		data.retain(|_, entry| now.duration_since(entry.window_start) < max_age * 2);
		*last_cleanup = now;
	}
}

impl Default for InMemoryStore {
	fn default() -> Self {
		Self::new()
	}
}

impl RateLimitStore for InMemoryStore {
	fn check_and_update(&self, key: &str, max_requests: u64, window: Duration) -> (bool, u64, u64) {
		self.cleanup(window);

		let now = Instant::now();
		let mut data = self.data.write().unwrap();
		let entry = data.entry(key.to_string()).or_default();
		let elapsed = now.duration_since(entry.window_start);

		if elapsed >= window {
			entry.prev_count = entry.count;
			entry.count = 0;
			entry.window_start = now;
		}

		entry.count += 1;

		let allowed = entry.count <= max_requests;
		let remaining = if allowed {
			max_requests - entry.count
		} else {
			0
		};

		let reset_at = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_secs()
			+ (window.as_secs() - elapsed.as_secs());

		(allowed, remaining, reset_at)
	}

	fn reset(&self, key: &str) {
		let mut data = self.data.write().unwrap();
		data.remove(key);
	}

	fn get_status(&self, key: &str, max_requests: u64, window: Duration) -> (u64, u64, u64) {
		let data = self.data.read().unwrap();

		match data.get(key) {
			Some(entry) => {
				let now = Instant::now();
				let elapsed = now.duration_since(entry.window_start);

				if elapsed >= window {
					let reset_at = SystemTime::now()
						.duration_since(UNIX_EPOCH)
						.unwrap()
						.as_secs() + window.as_secs();
					(0, max_requests, reset_at)
				} else {
					let remaining = max_requests.saturating_sub(entry.count);
					let reset_at = SystemTime::now()
						.duration_since(UNIX_EPOCH)
						.unwrap()
						.as_secs() + (window.as_secs() - elapsed.as_secs());
					(entry.count, remaining, reset_at)
				}
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
