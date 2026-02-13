use super::InMemoryStore;
use super::RateLimitAlgorithm;
use super::RateLimitConfig;
use super::RateLimitKey;
use super::RateLimitStore;
use super::SlidingWindowStore;
use super::TokenBucketStore;
use crate::mur_http::request::MurRequestContext;
use crate::traits::{MurMiddleware, MurNext};
use crate::types::{MurFuture, MurHttpResponse, MurRes};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct MurRateLimit {
	pub config: RateLimitConfig,
	pub store: Arc<dyn RateLimitStore>,
}

impl MurRateLimit {
	pub fn new() -> Self {
		Self {
			config: RateLimitConfig::default(),
			store: Arc::new(InMemoryStore::new()),
		}
	}

	pub fn from_config(config: RateLimitConfig) -> Self {
		let store: Arc<dyn RateLimitStore> = match config.algorithm {
			RateLimitAlgorithm::FixedWindow => Arc::new(InMemoryStore::new()),
			RateLimitAlgorithm::SlidingWindow => Arc::new(SlidingWindowStore::new()),
			RateLimitAlgorithm::TokenBucket => Arc::new(TokenBucketStore::new()),
		};

		Self { config, store }
	}

	pub fn requests(mut self, max: u64) -> Self {
		self.config.max_requests = max;
		self
	}

	pub fn per_seconds(mut self, seconds: u64) -> Self {
		self.config.window = Duration::from_secs(seconds);
		self
	}

	pub fn per_minutes(mut self, minutes: u64) -> Self {
		self.config.window = Duration::from_secs(minutes * 60);
		self
	}

	pub fn per_hours(mut self, hours: u64) -> Self {
		self.config.window = Duration::from_secs(hours * 3600);
		self
	}

	pub fn window(mut self, duration: Duration) -> Self {
		self.config.window = duration;
		self
	}

	pub fn by_ip(mut self) -> Self {
		self.config.key_extractor = RateLimitKey::Ip;
		self
	}

	pub fn by_header(mut self, name: impl Into<String>) -> Self {
		self.config.key_extractor = RateLimitKey::Header(name.into());
		self
	}

	pub fn by_bearer_token(mut self) -> Self {
		self.config.key_extractor = RateLimitKey::BearerToken;
		self
	}

	pub fn by_ip_and_header(mut self, name: impl Into<String>) -> Self {
		self.config.key_extractor = RateLimitKey::IpAndHeader(name.into());
		self
	}

	pub fn global(mut self) -> Self {
		self.config.key_extractor = RateLimitKey::Global;
		self
	}

	pub fn by_custom<F>(mut self, extractor: F) -> Self
	where
		F: Fn(&MurRequestContext) -> Option<String> + Send + Sync + 'static,
	{
		self.config.key_extractor = RateLimitKey::Custom(Arc::new(extractor));
		self
	}

	pub fn fixed_window(mut self) -> Self {
		self.config.algorithm = RateLimitAlgorithm::FixedWindow;
		self.store = Arc::new(InMemoryStore::new());
		self
	}

	pub fn sliding_window(mut self) -> Self {
		self.config.algorithm = RateLimitAlgorithm::SlidingWindow;
		self.store = Arc::new(SlidingWindowStore::new());
		self
	}

	pub fn token_bucket(mut self) -> Self {
		self.config.algorithm = RateLimitAlgorithm::TokenBucket;
		self.store = Arc::new(TokenBucketStore::new());
		self
	}

	pub fn with_store<S: RateLimitStore>(mut self, store: S) -> Self {
		self.store = Arc::new(store);
		self
	}

	pub fn message(mut self, msg: impl Into<String>) -> Self {
		self.config.message = Some(msg.into());
		self
	}

	pub fn no_headers(mut self) -> Self {
		self.config.include_headers = false;
		self
	}

	pub fn skip_path(mut self, path: impl Into<String>) -> Self {
		self.config.skip_paths.push(path.into());
		self
	}

	pub fn skip_paths(mut self, paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
		for path in paths {
			self.config.skip_paths.push(path.into());
		}
		self
	}

	pub fn skip_on_missing_key(mut self) -> Self {
		self.config.skip_on_missing_key = true;
		self
	}

	pub fn status_code(mut self, code: u16) -> Self {
		self.config.status_code = code;
		self
	}

	pub fn should_skip(&self, path: &str) -> bool {
		self.config.skip_paths.iter().any(|p| {
			if p.ends_with('*') {
				path.starts_with(&p[..p.len() - 1])
			} else {
				path == p
			}
		})
	}

	fn rate_limit_response(&self, _remaining: u64, reset_at: u64, retry_after: u64) -> MurRes {
		let message = self
			.config
			.message
			.clone()
			.unwrap_or_else(|| "Too Many Requests. Please try again later.".to_string());

		let status = http::StatusCode::from_u16(self.config.status_code)
			.unwrap_or(http::StatusCode::TOO_MANY_REQUESTS);

		let mut response = MurHttpResponse::status(status).json(serde_json::json!({
			"error": "Too Many Requests",
			"message": message,
			"retry_after": retry_after
		}))?;

		if self.config.include_headers {
			let headers = response.headers_mut();
			headers.insert(
				"X-RateLimit-Limit",
				self.config.max_requests.to_string().parse().unwrap(),
			);
			headers.insert("X-RateLimit-Remaining", "0".parse().unwrap());
			headers.insert("X-RateLimit-Reset", reset_at.to_string().parse().unwrap());
			headers.insert("Retry-After", retry_after.to_string().parse().unwrap());
		}

		Ok(response)
	}

	pub fn add_headers(
		&self,
		response: &mut http::Response<http_body_util::Full<hyper::body::Bytes>>,
		remaining: u64,
		reset_at: u64,
	) {
		if self.config.include_headers {
			let headers = response.headers_mut();
			headers.insert(
				"X-RateLimit-Limit",
				self.config.max_requests.to_string().parse().unwrap(),
			);
			headers.insert(
				"X-RateLimit-Remaining",
				remaining.to_string().parse().unwrap(),
			);
			headers.insert("X-RateLimit-Reset", reset_at.to_string().parse().unwrap());
		}
	}
}

impl Default for MurRateLimit {
	fn default() -> Self {
		Self::new()
	}
}

impl Clone for MurRateLimit {
	fn clone(&self) -> Self {
		Self {
			config: self.config.clone(),
			store: Arc::clone(&self.store),
		}
	}
}

impl std::fmt::Debug for MurRateLimit {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurRateLimit")
			.field("max_requests", &self.config.max_requests)
			.field("window", &self.config.window)
			.field("algorithm", &self.config.algorithm)
			.finish()
	}
}

impl MurMiddleware for MurRateLimit {
	fn handle(&self, ctx: MurRequestContext, next: MurNext) -> MurFuture {
		let path = ctx.path().to_string();

		if self.should_skip(&path) {
			return next.run(ctx);
		}

		let key = match self.config.key_extractor.extract(&ctx) {
			Some(k) => k,
			None => {
				if self.config.skip_on_missing_key {
					return next.run(ctx);
				}

				"unknown".to_string()
			}
		};

		let (allowed, remaining, reset_at) =
			self.store
				.check_and_update(&key, self.config.max_requests, self.config.window);

		if !allowed {
			let now = SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.unwrap()
				.as_secs();
			let retry_after = reset_at.saturating_sub(now);
			let response = self.rate_limit_response(remaining, reset_at, retry_after);

			return Box::pin(async move { response });
		}

		let include_headers = self.config.include_headers;
		let max_requests = self.config.max_requests;

		Box::pin(async move {
			let result = next.run(ctx).await;

			match result {
				Ok(mut response) => {
					if include_headers {
						let headers = response.headers_mut();
						headers.insert(
							"X-RateLimit-Limit",
							max_requests.to_string().parse().unwrap(),
						);
						headers.insert(
							"X-RateLimit-Remaining",
							remaining.to_string().parse().unwrap(),
						);
						headers.insert("X-RateLimit-Reset", reset_at.to_string().parse().unwrap());
					}
					Ok(response)
				}
				Err(e) => Err(e),
			}
		})
	}

	fn name(&self) -> &str {
		"MurRateLimit"
	}
}
