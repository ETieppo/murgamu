use super::server_variable::MurApiServerVariable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MurApiServer {
	pub url: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "HashMap::is_empty", default)]
	pub variables: HashMap<String, MurApiServerVariable>,
}
