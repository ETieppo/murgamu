use crate::middleware::health::indicator::MurHealthIndicator;
use crate::middleware::health::indicator_result::MurHealthIndicatorResult;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct MurDiskHealthIndicator {
	path: String,
	min_free_bytes: u64,
}

impl Default for MurDiskHealthIndicator {
	fn default() -> Self {
		Self {
			path: "/".to_string(),
			min_free_bytes: 100 * 1024 * 1024,
		}
	}
}

impl MurDiskHealthIndicator {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn path(mut self, path: impl Into<String>) -> Self {
		self.path = path.into();
		self
	}

	pub fn min_free_bytes(mut self, bytes: u64) -> Self {
		self.min_free_bytes = bytes;
		self
	}

	pub fn min_free_mb(self, mb: u64) -> Self {
		self.min_free_bytes(mb * 1024 * 1024)
	}

	pub fn min_free_gb(self, gb: u64) -> Self {
		self.min_free_bytes(gb * 1024 * 1024 * 1024)
	}
}

impl MurHealthIndicator for MurDiskHealthIndicator {
	fn check(&self) -> Pin<Box<dyn Future<Output = MurHealthIndicatorResult> + Send + '_>> {
		let path = self.path.clone();
		let min_free = self.min_free_bytes;

		Box::pin(async move {
			MurHealthIndicatorResult::healthy()
				.detail("path", path)
				.detail("min_free_bytes", min_free)
		})
	}

	fn name(&self) -> &str {
		"disk"
	}
}
