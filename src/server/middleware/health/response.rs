use super::MurHealthIndicatorResult;
use super::MurHealthStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MurHealthResponse {
	pub status: MurHealthStatus,

	#[serde(skip_serializing_if = "HashMap::is_empty")]
	pub indicators: HashMap<String, MurHealthIndicatorResult>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub timestamp: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub total_duration_ms: Option<u64>,
}

impl MurHealthResponse {
	pub fn healthy() -> Self {
		Self {
			status: MurHealthStatus::Up,
			indicators: HashMap::new(),
			version: None,
			timestamp: Some(chrono::Utc::now().to_rfc3339()),
			total_duration_ms: None,
		}
	}

	pub fn unhealthy() -> Self {
		Self {
			status: MurHealthStatus::Down,
			indicators: HashMap::new(),
			version: None,
			timestamp: Some(chrono::Utc::now().to_rfc3339()),
			total_duration_ms: None,
		}
	}
}
