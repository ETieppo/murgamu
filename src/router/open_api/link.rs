use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiLink {
	#[serde(rename = "operationRef", skip_serializing_if = "Option::is_none")]
	pub operation_ref: Option<String>,
	#[serde(rename = "operationId", skip_serializing_if = "Option::is_none")]
	pub operation_id: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
}
