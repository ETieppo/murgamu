use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MurApiServerVariable {
	pub default: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(rename = "enum", skip_serializing_if = "Vec::is_empty", default)]
	pub enum_values: Vec<String>,
}
