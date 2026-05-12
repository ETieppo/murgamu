use super::MurThrottlerEntry;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub trait MurThrottlerStore: Send + Sync + 'static {
	fn check_and_update(&self, key: &str, max_requests: u64, window: Duration) -> (bool, u64, u64);
	fn reset(&self, key: &str);
	fn get_status(&self, key: &str, max_requests: u64, window: Duration) -> (u64, u64, u64);
}

#[derive(Debug)]
pub struct InMemoryStore {
	pub data: RwLock<HashMap<String, MurThrottlerEntry>>,
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
		// Fast path: read lock to avoid write contention on every request.
		{
			let last = self.last_cleanup.read().unwrap_or_else(|e| e.into_inner());
			if last.elapsed() < self.cleanup_interval {
				return;
			}
		}
		// Double-check under write lock to prevent multiple concurrent cleanups.
		let mut last_cleanup = self.last_cleanup.write().unwrap_or_else(|e| e.into_inner());
		if last_cleanup.elapsed() < self.cleanup_interval {
			return;
		}
		let now = Instant::now();
		*last_cleanup = now;
		drop(last_cleanup); // Release before acquiring data lock to avoid deadlock.

		let mut data = self.data.write().unwrap_or_else(|e| e.into_inner());
		data.retain(|_, entry| now.duration_since(entry.window_start) < max_age * 2);
	}
}

impl Default for InMemoryStore {
	fn default() -> Self {
		Self::new()
	}
}

impl MurThrottlerStore for InMemoryStore {
	fn check_and_update(&self, key: &str, max_requests: u64, window: Duration) -> (bool, u64, u64) {
		self.cleanup(window);

		let now = Instant::now();
		let mut data = self.data.write().unwrap_or_else(|e| e.into_inner());
		let entry = data.entry(key.to_string()).or_default();
		let elapsed = now.duration_since(entry.window_start);

		let remaining_secs = if elapsed >= window {
			entry.prev_count = entry.count;
			entry.count = 0;
			entry.window_start = now;
			window.as_secs()
		} else {
			window.as_secs().saturating_sub(elapsed.as_secs())
		};

		entry.count += 1;

		let allowed = entry.count <= max_requests;
		let remaining = if allowed { max_requests - entry.count } else { 0 };
		let reset_at = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs()
			+ remaining_secs;

		(allowed, remaining, reset_at)
	}

	fn reset(&self, key: &str) {
		let mut data = self.data.write().unwrap_or_else(|e| e.into_inner());
		data.remove(key);
	}

	fn get_status(&self, key: &str, max_requests: u64, window: Duration) -> (u64, u64, u64) {
		let data = self.data.read().unwrap_or_else(|e| e.into_inner());
		let now_unix = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();

		match data.get(key) {
			Some(entry) => {
				let now = Instant::now();
				let elapsed = now.duration_since(entry.window_start);

				if elapsed >= window {
					(0, max_requests, now_unix + window.as_secs())
				} else {
					let remaining = max_requests.saturating_sub(entry.count);
					let reset_at = now_unix + window.as_secs().saturating_sub(elapsed.as_secs());
					(entry.count, remaining, reset_at)
				}
			}
			None => (0, max_requests, now_unix + window.as_secs()),
		}
	}
}
