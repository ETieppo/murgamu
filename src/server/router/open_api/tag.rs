use super::external_doc::MurApiExternalDocs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MurApiTag {
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(rename = "externalDocs", skip_serializing_if = "Option::is_none")]
	pub external_docs: Option<MurApiExternalDocs>,
}

impl MurApiTag {
	pub fn new(name: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			description: None,
			external_docs: None,
		}
	}

	pub fn description(mut self, desc: impl Into<String>) -> Self {
		self.description = Some(desc.into());
		self
	}
}
