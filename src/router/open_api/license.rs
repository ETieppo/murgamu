use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MurApiLicense {
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub url: Option<String>,
}
