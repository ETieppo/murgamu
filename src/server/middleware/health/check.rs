use super::MurHealthBuilder;
use super::MurHealthConfig;
use super::MurHealthIndicator;
use super::MurHealthResponse;
use super::MurHealthStatus;
use crate::server::aliases::MurRes;
use crate::server::http::MurHttpResponse;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::task::JoinSet;

#[derive(Clone)]
pub struct MurHealthCheck {
	pub config: Arc<MurHealthConfig>,
	pub indicators: Arc<Vec<(String, Arc<dyn MurHealthIndicator + Send + Sync>)>>,
	pub readiness_indicators: Arc<Vec<String>>,
}

impl MurHealthCheck {
	pub fn new() -> Self {
		MurHealthBuilder::new().build()
	}

	pub fn builder() -> MurHealthBuilder {
		MurHealthBuilder::new()
	}

	pub fn path(&self) -> &str {
		&self.config.path
	}

	pub fn liveness_path(&self) -> Option<&str> {
		self.config.liveness_path.as_deref()
	}

	pub fn readiness_path(&self) -> Option<&str> {
		self.config.readiness_path.as_deref()
	}

	pub async fn check(&self) -> MurHealthResponse {
		self.run_indicators(self.indicators.as_ref()).await
	}

	pub async fn check_liveness(&self) -> MurHealthResponse {
		MurHealthResponse::healthy()
	}

	pub async fn check_readiness(&self) -> MurHealthResponse {
		if self.readiness_indicators.is_empty() {
			return self.check().await;
		}

		let indicators: Vec<_> = self
			.indicators
			.iter()
			.filter(|(name, _)| self.readiness_indicators.contains(name))
			.cloned()
			.collect();

		self.run_indicators(&indicators).await
	}

	async fn run_indicators(
		&self,
		indicators: &[(String, Arc<dyn MurHealthIndicator + Send + Sync>)],
	) -> MurHealthResponse {
		let start = Instant::now();
		let mut response = MurHealthResponse::healthy();
		let mut results = HashMap::new();
		let mut overall_status = MurHealthStatus::Up;
		response.version = self.config.version.clone();

		if !self.config.include_timestamp {
			response.timestamp = None;
		}

		if indicators.is_empty() {
			if self.config.include_duration {
				response.total_duration_ms = Some(start.elapsed().as_millis() as u64);
			}
			return response;
		}

		if self.config.parallel {
			let mut set = JoinSet::new();

			for (name, indicator) in indicators.iter() {
				let name = name.clone();
				let indicator = Arc::clone(indicator);
				let include_duration = self.config.include_duration;

				set.spawn(async move {
					let indicator_start = Instant::now();
					let mut result = indicator.check().await;

					if include_duration {
						result.duration_ms = Some(indicator_start.elapsed().as_millis() as u64);
					}

					(name, result)
				});
			}

			while let Some(Ok((name, result))) = set.join_next().await {
				overall_status = overall_status.combine(result.status);
				results.insert(name, result);
			}
		} else {
			for (name, indicator) in indicators {
				let indicator_start = Instant::now();
				let mut result = indicator.check().await;
				if self.config.include_duration {
					result.duration_ms = Some(indicator_start.elapsed().as_millis() as u64);
				}
				overall_status = overall_status.combine(result.status);
				results.insert(name.clone(), result);
			}
		}

		response.status = overall_status;

		if self.config.include_details {
			response.indicators = results;
		}

		if self.config.include_duration {
			response.total_duration_ms = Some(start.elapsed().as_millis() as u64);
		}

		response
	}

	pub async fn handle_request(&self, path: &str) -> MurRes {
		use hyper::StatusCode;

		let response = if Some(path) == self.liveness_path() {
			self.check_liveness().await
		} else if Some(path) == self.readiness_path() {
			self.check_readiness().await
		} else {
			self.check().await
		};

		let status_code = response.status.http_status_code();
		let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
		MurHttpResponse::status(status).json(response)
	}
}

impl Default for MurHealthCheck {
	fn default() -> Self {
		Self::new()
	}
}

impl std::fmt::Debug for MurHealthCheck {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurHealthCheck")
			.field("config", &self.config)
			.field("indicators_count", &self.indicators.len())
			.finish()
	}
}
