use crate::middleware::health::indicator::MurHealthIndicator;
use crate::middleware::health::indicator_result::MurHealthIndicatorResult;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub struct MurCustomIndicator<F>
where
	F: Fn() -> Pin<Box<dyn Future<Output = MurHealthIndicatorResult> + Send>> + Send + Sync,
{
	name: String,
	check_fn: F,
	timeout: Duration,
}

impl<F> MurCustomIndicator<F>
where
	F: Fn() -> Pin<Box<dyn Future<Output = MurHealthIndicatorResult> + Send>> + Send + Sync,
{
	pub fn new(name: impl Into<String>, check_fn: F) -> Self {
		Self {
			name: name.into(),
			check_fn,
			timeout: Duration::from_secs(5),
		}
	}

	pub fn timeout(mut self, timeout: Duration) -> Self {
		self.timeout = timeout;
		self
	}
}

impl<F> MurHealthIndicator for MurCustomIndicator<F>
where
	F: Fn() -> Pin<Box<dyn Future<Output = MurHealthIndicatorResult> + Send>>
		+ Send
		+ Sync
		+ 'static,
{
	fn check(&self) -> Pin<Box<dyn Future<Output = MurHealthIndicatorResult> + Send + '_>> {
		(self.check_fn)()
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn timeout(&self) -> Duration {
		self.timeout
	}
}
