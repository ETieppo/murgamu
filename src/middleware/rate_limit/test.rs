use super::*;
use std::time::Duration;

pub fn mur_rate_limit_per_minute(requests: u64) -> MurRateLimit {
	MurRateLimit::new().requests(requests).per_minutes(1)
}

pub fn mur_rate_limit_per_hour(requests: u64) -> MurRateLimit {
	MurRateLimit::new().requests(requests).per_hours(1)
}

pub fn mur_rate_limit_strict() -> MurRateLimit {
	MurRateLimit::new().requests(10).per_minutes(1).by_ip()
}

pub fn mur_rate_limit_permissive() -> MurRateLimit {
	MurRateLimit::new().requests(1000).per_minutes(1).by_ip()
}

#[test]
fn test_config_defaults() {
	let config = RateLimitConfig::default();
	assert_eq!(config.max_requests, 100);
	assert_eq!(config.window, Duration::from_secs(60));
	assert!(config.include_headers);
	assert!(config.skip_paths.is_empty());
}

#[test]
fn test_rate_limit_builder() {
	let limiter = MurRateLimit::new().requests(500).per_minutes(5).by_ip();

	assert_eq!(limiter.config.max_requests, 500);
	assert_eq!(limiter.config.window, Duration::from_secs(300));
}

#[test]
fn test_in_memory_store_basic() {
	let store = InMemoryStore::new();
	let window = Duration::from_secs(60);

	let (allowed, remaining, _) = store.check_and_update("test_key", 10, window);
	assert!(allowed);
	assert_eq!(remaining, 9);

	for i in 0..9 {
		let (allowed, remaining, _) = store.check_and_update("test_key", 10, window);
		assert!(allowed, "Request {} should be allowed", i + 2);
		assert_eq!(remaining, 8 - i);
	}

	let (allowed, remaining, _) = store.check_and_update("test_key", 10, window);
	assert!(!allowed);
	assert_eq!(remaining, 0);
}

#[test]
fn test_in_memory_store_different_keys() {
	let store = InMemoryStore::new();
	let window = Duration::from_secs(60);

	let (allowed, _, _) = store.check_and_update("key_a", 5, window);
	assert!(allowed);

	let (allowed, remaining, _) = store.check_and_update("key_b", 5, window);
	assert!(allowed);
	assert_eq!(remaining, 4);
}

#[test]
fn test_token_bucket_store() {
	let store = TokenBucketStore::new();
	let window = Duration::from_secs(60);

	let (allowed, remaining, _) = store.check_and_update("test", 10, window);
	assert!(allowed);
	assert_eq!(remaining, 9);

	for _ in 0..8 {
		let (allowed, _, _) = store.check_and_update("test", 10, window);
		assert!(allowed);
	}

	let (allowed, remaining, _) = store.check_and_update("test", 10, window);
	assert!(allowed);
	assert_eq!(remaining, 0);

	let (allowed, _, _) = store.check_and_update("test", 10, window);
	assert!(!allowed);
}

#[test]
fn test_skip_paths() {
	let limiter = MurRateLimit::new()
		.skip_path("/health")
		.skip_path("/metrics")
		.skip_path("/api/public/*");

	assert!(limiter.should_skip("/health"));
	assert!(limiter.should_skip("/metrics"));
	assert!(limiter.should_skip("/api/public/data"));
	assert!(limiter.should_skip("/api/public/other"));
	assert!(!limiter.should_skip("/api/private"));
	assert!(!limiter.should_skip("/users"));
}

#[test]
fn test_rate_limit_key_extraction() {
	let _key = RateLimitKey::Ip;

	let _key = RateLimitKey::Global;
}

#[test]
fn test_sliding_window_store() {
	let store = SlidingWindowStore::new();
	let window = Duration::from_secs(60);
	let (allowed, _, _) = store.check_and_update("test", 10, window);
	assert!(allowed);
}

#[test]
fn test_convenience_functions() {
	let limiter = mur_rate_limit_per_minute(100);
	assert_eq!(limiter.config.max_requests, 100);
	assert_eq!(limiter.config.window, Duration::from_secs(60));

	let limiter = mur_rate_limit_per_hour(1000);
	assert_eq!(limiter.config.max_requests, 1000);
	assert_eq!(limiter.config.window, Duration::from_secs(3600));

	let limiter = mur_rate_limit_strict();
	assert_eq!(limiter.config.max_requests, 10);

	let limiter = mur_rate_limit_permissive();
	assert_eq!(limiter.config.max_requests, 1000);
}

#[test]
fn test_rate_limit_algorithms() {
	let limiter = MurRateLimit::new().fixed_window();
	assert_eq!(limiter.config.algorithm, RateLimitAlgorithm::FixedWindow);

	let limiter = MurRateLimit::new().sliding_window();
	assert_eq!(limiter.config.algorithm, RateLimitAlgorithm::SlidingWindow);

	let limiter = MurRateLimit::new().token_bucket();
	assert_eq!(limiter.config.algorithm, RateLimitAlgorithm::TokenBucket);
}

#[test]
fn test_rate_limit_clone() {
	let limiter = MurRateLimit::new().requests(100).per_minutes(1);
	let cloned = limiter.clone();
	assert_eq!(cloned.config.max_requests, 100);
}

#[test]
fn test_store_reset() {
	let store = InMemoryStore::new();
	let window = Duration::from_secs(60);

	store.check_and_update("test", 10, window);
	store.check_and_update("test", 10, window);

	let (count, _, _) = store.get_status("test", 10, window);
	assert_eq!(count, 2);
	store.reset("test");

	let (count, remaining, _) = store.get_status("test", 10, window);
	assert_eq!(count, 0);
	assert_eq!(remaining, 10);
}
