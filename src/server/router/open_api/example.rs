use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiExample {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub summary: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub value: Option<serde_json::Value>,
	#[serde(rename = "externalValue", skip_serializing_if = "Option::is_none")]
	pub external_value: Option<String>,
}
