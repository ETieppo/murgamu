use super::schema::MurApiSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MurApiParameterIn {
	Query,
	Header,
	Path,
	Cookie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MurApiParameter {
	pub name: String,
	#[serde(rename = "in")]
	pub location: MurApiParameterIn,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub required: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub deprecated: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub schema: Option<MurApiSchema>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub example: Option<serde_json::Value>,
}

impl MurApiParameter {
	pub fn path(name: impl Into<String>, description: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			location: MurApiParameterIn::Path,
			description: Some(description.into()),
			required: Some(true),
			deprecated: None,
			schema: Some(MurApiSchema::string()),
			example: None,
		}
	}

	pub fn query(name: impl Into<String>, description: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			location: MurApiParameterIn::Query,
			description: Some(description.into()),
			required: None,
			deprecated: None,
			schema: Some(MurApiSchema::string()),
			example: None,
		}
	}

	pub fn header(name: impl Into<String>, description: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			location: MurApiParameterIn::Header,
			description: Some(description.into()),
			required: None,
			deprecated: None,
			schema: Some(MurApiSchema::string()),
			example: None,
		}
	}

	pub fn cookie(name: impl Into<String>, description: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			location: MurApiParameterIn::Cookie,
			description: Some(description.into()),
			required: None,
			deprecated: None,
			schema: Some(MurApiSchema::string()),
			example: None,
		}
	}

	pub fn required(mut self) -> Self {
		self.required = Some(true);
		self
	}

	pub fn optional(mut self) -> Self {
		self.required = Some(false);
		self
	}

	pub fn schema(mut self, schema: MurApiSchema) -> Self {
		self.schema = Some(schema);
		self
	}

	pub fn example(mut self, example: impl Into<serde_json::Value>) -> Self {
		self.example = Some(example.into());
		self
	}

	pub fn deprecated(mut self) -> Self {
		self.deprecated = Some(true);
		self
	}
}
