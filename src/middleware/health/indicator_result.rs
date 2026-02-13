use crate::middleware::health::status::MurHealthStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MurHealthIndicatorResult {
	pub status: MurHealthStatus,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub details: Option<HashMap<String, serde_json::Value>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub duration_ms: Option<u64>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub error: Option<String>,
}

impl MurHealthIndicatorResult {
	pub fn healthy() -> Self {
		Self {
			status: MurHealthStatus::Up,
			details: None,
			duration_ms: None,
			error: None,
		}
	}

	pub fn unhealthy() -> Self {
		Self {
			status: MurHealthStatus::Down,
			details: None,
			duration_ms: None,
			error: None,
		}
	}

	pub fn degraded() -> Self {
		Self {
			status: MurHealthStatus::Degraded,
			details: None,
			duration_ms: None,
			error: None,
		}
	}

	pub fn with_error(error: impl Into<String>) -> Self {
		Self {
			status: MurHealthStatus::Down,
			details: None,
			duration_ms: None,
			error: Some(error.into()),
		}
	}

	pub fn detail(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
		let details = self.details.get_or_insert_with(HashMap::new);
		if let Ok(v) = serde_json::to_value(value) {
			details.insert(key.into(), v);
		}
		self
	}

	pub fn duration(mut self, duration: Duration) -> Self {
		self.duration_ms = Some(duration.as_millis() as u64);
		self
	}

	pub fn error(mut self, error: impl Into<String>) -> Self {
		self.error = Some(error.into());
		self.status = MurHealthStatus::Down;
		self
	}

	pub fn status(mut self, status: MurHealthStatus) -> Self {
		self.status = status;
		self
	}
}
