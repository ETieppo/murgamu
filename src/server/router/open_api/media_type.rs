use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{example::MurApiExample, schema::MurApiSchema};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiMediaType {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub schema: Option<MurApiSchema>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub example: Option<serde_json::Value>,
	#[serde(skip_serializing_if = "HashMap::is_empty", default)]
	pub examples: HashMap<String, MurApiExample>,
}
