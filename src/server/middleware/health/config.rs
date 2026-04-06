use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MurHealthConfig {
	pub path: String,
	pub liveness_path: Option<String>,
	pub readiness_path: Option<String>,
	pub include_details: bool,
	pub include_timestamp: bool,
	pub include_duration: bool,
	pub version: Option<String>,
	pub timeout: Duration,
	pub parallel: bool,
}

impl Default for MurHealthConfig {
	fn default() -> Self {
		Self {
			path: "/health".to_string(),
			liveness_path: Some("/health/live".to_string()),
			readiness_path: Some("/health/ready".to_string()),
			include_details: true,
			include_timestamp: true,
			include_duration: true,
			version: None,
			timeout: Duration::from_secs(10),
			parallel: true,
		}
	}
}
