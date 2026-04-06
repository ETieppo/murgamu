use super::schema::MurApiSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiHeader {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub schema: Option<MurApiSchema>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub required: Option<bool>,
}
