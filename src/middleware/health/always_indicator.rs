use crate::middleware::health::indicator::MurHealthIndicator;
use crate::middleware::health::indicator_result::MurHealthIndicatorResult;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Default)]
pub struct MurAlwaysHealthyIndicator;

impl MurHealthIndicator for MurAlwaysHealthyIndicator {
	fn check(&self) -> Pin<Box<dyn Future<Output = MurHealthIndicatorResult> + Send + '_>> {
		Box::pin(async { MurHealthIndicatorResult::healthy() })
	}

	fn name(&self) -> &str {
		"liveness"
	}
}
