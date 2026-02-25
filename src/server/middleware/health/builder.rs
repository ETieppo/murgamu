use super::MurCustomIndicator;
use super::MurHealthCheck;
use super::MurHealthConfig;
use super::MurHealthIndicator;
use super::MurHealthIndicatorResult;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

pub struct MurHealthBuilder {
	config: MurHealthConfig,
	indicators: Vec<(String, Arc<dyn MurHealthIndicator + Send + Sync>)>,
	readiness_indicators: Vec<String>,
}

impl MurHealthBuilder {
	pub fn new() -> Self {
		Self {
			config: MurHealthConfig::default(),
			indicators: Vec::new(),
			readiness_indicators: Vec::new(),
		}
	}

	pub fn path(mut self, path: impl Into<String>) -> Self {
		self.config.path = path.into();
		self
	}

	pub fn liveness_path(mut self, path: impl Into<String>) -> Self {
		self.config.liveness_path = Some(path.into());
		self
	}

	pub fn readiness_path(mut self, path: impl Into<String>) -> Self {
		self.config.readiness_path = Some(path.into());
		self
	}

	pub fn no_liveness(mut self) -> Self {
		self.config.liveness_path = None;
		self
	}

	pub fn no_readiness(mut self) -> Self {
		self.config.readiness_path = None;
		self
	}

	pub fn include_details(mut self, include: bool) -> Self {
		self.config.include_details = include;
		self
	}

	pub fn include_timestamp(mut self, include: bool) -> Self {
		self.config.include_timestamp = include;
		self
	}

	pub fn include_duration(mut self, include: bool) -> Self {
		self.config.include_duration = include;
		self
	}

	pub fn version(mut self, version: impl Into<String>) -> Self {
		self.config.version = Some(version.into());
		self
	}

	pub fn timeout(mut self, timeout: Duration) -> Self {
		self.config.timeout = timeout;
		self
	}

	pub fn parallel(mut self, parallel: bool) -> Self {
		self.config.parallel = parallel;
		self
	}

	pub fn indicator<I: MurHealthIndicator>(
		mut self,
		name: impl Into<String>,
		indicator: I,
	) -> Self {
		let name = name.into();
		self.indicators.push((name, Arc::new(indicator)));
		self
	}

	pub fn readiness_indicator<I: MurHealthIndicator>(
		mut self,
		name: impl Into<String>,
		indicator: I,
	) -> Self {
		let name = name.into();
		self.readiness_indicators.push(name.clone());
		self.indicators.push((name, Arc::new(indicator)));
		self
	}

	pub fn check<F>(self, name: impl Into<String>, check_fn: F) -> Self
	where
		F: Fn() -> Pin<Box<dyn Future<Output = MurHealthIndicatorResult> + Send>>
			+ Send
			+ Sync
			+ 'static,
	{
		let name_str: String = name.into();
		let indicator = MurCustomIndicator::new(name_str.clone(), check_fn);
		self.indicator(name_str, indicator)
	}

	pub fn build(self) -> MurHealthCheck {
		MurHealthCheck {
			config: Arc::new(self.config),
			indicators: Arc::new(self.indicators),
			readiness_indicators: Arc::new(self.readiness_indicators),
		}
	}
}

impl Default for MurHealthBuilder {
	fn default() -> Self {
		Self::new()
	}
}
