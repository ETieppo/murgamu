use super::{media_type::MurApiMediaType, schema::MurApiSchema};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiRequestBody {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	pub content: IndexMap<String, MurApiMediaType>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub required: Option<bool>,
}

impl MurApiRequestBody {
	pub fn json(schema: MurApiSchema) -> Self {
		let mut content = IndexMap::new();
		content.insert(
			"application/json".to_string(),
			MurApiMediaType {
				schema: Some(schema),
				example: None,
				examples: HashMap::new(),
			},
		);
		Self {
			description: None,
			content,
			required: Some(true),
		}
	}

	pub fn form(schema: MurApiSchema) -> Self {
		let mut content = IndexMap::new();
		content.insert(
			"application/x-www-form-urlencoded".to_string(),
			MurApiMediaType {
				schema: Some(schema),
				example: None,
				examples: HashMap::new(),
			},
		);
		Self {
			description: None,
			content,
			required: Some(true),
		}
	}

	pub fn multipart(schema: MurApiSchema) -> Self {
		let mut content = IndexMap::new();
		content.insert(
			"multipart/form-data".to_string(),
			MurApiMediaType {
				schema: Some(schema),
				example: None,
				examples: HashMap::new(),
			},
		);
		Self {
			description: None,
			content,
			required: Some(true),
		}
	}

	pub fn description(mut self, desc: impl Into<String>) -> Self {
		self.description = Some(desc.into());
		self
	}

	pub fn optional(mut self) -> Self {
		self.required = Some(false);
		self
	}
}
