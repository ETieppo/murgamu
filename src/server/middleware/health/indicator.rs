use super::MurHealthIndicatorResult;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub trait MurHealthIndicator: Send + Sync + 'static {
	fn check(&self) -> Pin<Box<dyn Future<Output = MurHealthIndicatorResult> + Send + '_>>;

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}

	fn timeout(&self) -> Duration {
		Duration::from_secs(5)
	}
}
