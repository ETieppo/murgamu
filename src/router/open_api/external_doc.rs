use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MurApiExternalDocs {
	pub url: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
}
