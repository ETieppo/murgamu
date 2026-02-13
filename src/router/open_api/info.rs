use serde::{Deserialize, Serialize};
use super::{contact::MurApiContact, license::MurApiLicense};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MurApiInfo {
	pub title: String,
	pub version: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(rename = "termsOfService", skip_serializing_if = "Option::is_none")]
	pub terms_of_service: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub contact: Option<MurApiContact>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub license: Option<MurApiLicense>,
}

impl Default for MurApiInfo {
	fn default() -> Self {
		Self {
			title: "API".to_string(),
			version: "1.0.0".to_string(),
			description: None,
			terms_of_service: None,
			contact: None,
			license: None,
		}
	}
}
