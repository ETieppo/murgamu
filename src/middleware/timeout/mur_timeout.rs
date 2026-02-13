use super::TimeoutConfig;
use crate::mur_http::request::MurRequestContext;
use crate::traits::{MurMiddleware, MurNext};
use crate::types::{MurFuture, MurHttpResponse};
use hyper::StatusCode;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct MurTimeout {
	pub config: Arc<TimeoutConfig>,
}

impl MurTimeout {
	pub fn new(timeout: Duration) -> Self {
		Self {
			config: Arc::new(TimeoutConfig::new(timeout)),
		}
	}

	pub fn from_secs(secs: u64) -> Self {
		Self::new(Duration::from_secs(secs))
	}

	pub fn from_millis(millis: u64) -> Self {
		Self::new(Duration::from_millis(millis))
	}

	pub fn from_config(config: TimeoutConfig) -> Self {
		Self {
			config: Arc::new(config),
		}
	}

	pub fn with_status(mut self, status: StatusCode) -> Self {
		let mut config = (*self.config).clone();
		config.status_code = status;
		self.config = Arc::new(config);
		self
	}

	pub fn request_timeout(self) -> Self {
		self.with_status(StatusCode::REQUEST_TIMEOUT)
	}

	pub fn gateway_timeout(self) -> Self {
		self.with_status(StatusCode::GATEWAY_TIMEOUT)
	}

	pub fn service_unavailable(self) -> Self {
		self.with_status(StatusCode::SERVICE_UNAVAILABLE)
	}

	pub fn with_message(mut self, message: impl Into<String>) -> Self {
		let mut config = (*self.config).clone();
		config.message = Some(message.into());
		self.config = Arc::new(config);
		self
	}

	pub fn skip_paths(mut self, paths: Vec<impl Into<String>>) -> Self {
		let mut config = (*self.config).clone();
		config.skip_paths = paths.into_iter().map(|p| p.into()).collect();
		self.config = Arc::new(config);
		self
	}

	pub fn skip_path(mut self, path: impl Into<String>) -> Self {
		let mut config = (*self.config).clone();
		config.skip_paths.push(path.into());
		self.config = Arc::new(config);
		self
	}

	pub fn skip_prefixes(mut self, prefixes: Vec<impl Into<String>>) -> Self {
		let mut config = (*self.config).clone();
		config.skip_path_prefixes = prefixes.into_iter().map(|p| p.into()).collect();
		self.config = Arc::new(config);
		self
	}

	pub fn skip_prefix(mut self, prefix: impl Into<String>) -> Self {
		let mut config = (*self.config).clone();
		config.skip_path_prefixes.push(prefix.into());
		self.config = Arc::new(config);
		self
	}

	pub fn include_header(mut self) -> Self {
		let mut config = (*self.config).clone();
		config.include_timeout_header = true;
		self.config = Arc::new(config);
		self
	}

	pub fn log(mut self, enable: bool) -> Self {
		let mut config = (*self.config).clone();
		config.log_timeouts = enable;
		self.config = Arc::new(config);
		self
	}

	pub fn should_skip(&self, path: &str) -> bool {
		if self.config.skip_paths.iter().any(|p| p == path) {
			return true;
		}

		if self
			.config
			.skip_path_prefixes
			.iter()
			.any(|p| path.starts_with(p.as_str()))
		{
			return true;
		}

		false
	}

	pub fn build_timeout_response(&self) -> crate::types::MurRes {
		let message = self
			.config
			.message
			.clone()
			.unwrap_or_else(|| "Request timed out".to_string());

		let timeout_secs = self.config.timeout.as_secs_f64();

		MurHttpResponse::status(self.config.status_code).json(serde_json::json!({
			"error": "Timeout",
			"message": message,
			"timeout_seconds": timeout_secs,
			"status": self.config.status_code.as_u16()
		}))
	}
}

impl Default for MurTimeout {
	fn default() -> Self {
		Self::new(Duration::from_secs(30))
	}
}

impl std::fmt::Debug for MurTimeout {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurTimeout")
			.field("config", &self.config)
			.finish()
	}
}

impl MurMiddleware for MurTimeout {
	fn handle(&self, ctx: MurRequestContext, next: MurNext) -> MurFuture {
		let config = Arc::clone(&self.config);
		let timeout_middleware = self.clone();

		Box::pin(async move {
			let path = ctx.path().to_string();
			let method = ctx.method().to_string();

			if timeout_middleware.should_skip(&path) {
				return next.run(ctx).await;
			}

			let start = std::time::Instant::now();
			let result = tokio::time::timeout(config.timeout, next.run(ctx)).await;

			match result {
				Ok(response) => {
					if config.include_timeout_header {
						if let Ok(mut resp) = response {
							let elapsed = start.elapsed();
							if let Ok(value) = format!("{}ms", elapsed.as_millis())
								.parse::<hyper::header::HeaderValue>()
							{
								if let Ok(header_name) = hyper::header::HeaderName::from_bytes(
									config.timeout_header_name.as_bytes(),
								) {
									resp.headers_mut().insert(header_name, value);
								}
							}
							return Ok(resp);
						}
					}
					response
				}
				Err(_elapsed) => {
					if config.log_timeouts {
						eprintln!(
							"[TIMEOUT] {} {} exceeded {}ms timeout",
							method,
							path,
							config.timeout.as_millis()
						);
					}
					timeout_middleware.build_timeout_response()
				}
			}
		})
	}

	fn name(&self) -> &str {
		"MurTimeout"
	}
}
