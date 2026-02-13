use super::{
	header::MurApiHeader, link::MurApiLink, media_type::MurApiMediaType, schema::MurApiSchema,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiResponse {
	pub description: String,
	#[serde(skip_serializing_if = "HashMap::is_empty", default)]
	pub headers: HashMap<String, MurApiHeader>,
	#[serde(skip_serializing_if = "IndexMap::is_empty", default)]
	pub content: IndexMap<String, MurApiMediaType>,
	#[serde(skip_serializing_if = "HashMap::is_empty", default)]
	pub links: HashMap<String, MurApiLink>,
}

impl MurApiResponse {
	pub fn new(description: impl Into<String>) -> Self {
		Self {
			description: description.into(),
			..Default::default()
		}
	}

	pub fn json(mut self, schema: MurApiSchema) -> Self {
		self.content.insert(
			"application/json".to_string(),
			MurApiMediaType {
				schema: Some(schema),
				example: None,
				examples: HashMap::new(),
			},
		);
		self
	}

	pub fn header(mut self, name: impl Into<String>, header: MurApiHeader) -> Self {
		self.headers.insert(name.into(), header);
		self
	}
}
