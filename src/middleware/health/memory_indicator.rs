use std::future::Future;
use std::pin::Pin;
use crate::middleware::health::indicator::MurHealthIndicator;
use crate::middleware::health::indicator_result::MurHealthIndicatorResult;

#[derive(Debug, Clone)]
pub struct MurMemoryHealthIndicator {
	degraded_threshold: f64,
	unhealthy_threshold: f64,
}

impl Default for MurMemoryHealthIndicator {
	fn default() -> Self {
		Self {
			degraded_threshold: 0.80,
			unhealthy_threshold: 0.95,
		}
	}
}

impl MurMemoryHealthIndicator {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn degraded_threshold(mut self, threshold: f64) -> Self {
		self.degraded_threshold = threshold.clamp(0.0, 1.0);
		self
	}

	pub fn unhealthy_threshold(mut self, threshold: f64) -> Self {
		self.unhealthy_threshold = threshold.clamp(0.0, 1.0);
		self
	}
}

impl MurHealthIndicator for MurMemoryHealthIndicator {
	fn check(&self) -> Pin<Box<dyn Future<Output = MurHealthIndicatorResult> + Send + '_>> {
		let degraded = self.degraded_threshold;
		let unhealthy = self.unhealthy_threshold;

		Box::pin(async move {
			let result = MurHealthIndicatorResult::healthy()
				.detail("degraded_threshold", format!("{}%", degraded * 100.0))
				.detail("unhealthy_threshold", format!("{}%", unhealthy * 100.0));

			result
		})
	}

	fn name(&self) -> &str {
		"memory"
	}
}
