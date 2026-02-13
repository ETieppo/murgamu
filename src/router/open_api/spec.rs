use super::{
	components::MurApiComponents, external_doc::MurApiExternalDocs, info::MurApiInfo,
	path_item::MurApiPathItem, server::MurApiServer, tag::MurApiTag,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MurOpenApiSpec {
	pub openapi: String,
	pub info: MurApiInfo,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub servers: Vec<MurApiServer>,
	#[serde(skip_serializing_if = "IndexMap::is_empty", default)]
	pub paths: IndexMap<String, MurApiPathItem>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub components: Option<MurApiComponents>,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub security: Vec<HashMap<String, Vec<String>>>,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub tags: Vec<MurApiTag>,
	#[serde(rename = "externalDocs", skip_serializing_if = "Option::is_none")]
	pub external_docs: Option<MurApiExternalDocs>,
}

impl Default for MurOpenApiSpec {
	fn default() -> Self {
		Self {
			openapi: "3.0.3".to_string(),
			info: MurApiInfo::default(),
			servers: Vec::new(),
			paths: IndexMap::new(),
			components: None,
			security: Vec::new(),
			tags: Vec::new(),
			external_docs: None,
		}
	}
}

impl MurOpenApiSpec {
	pub fn to_json(&self) -> Result<String, serde_json::Error> {
		serde_json::to_string_pretty(self)
	}

	pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
		serde_json::to_string(self)
	}
}
