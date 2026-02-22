use super::{operation::MurApiOperation, parameter::MurApiParameter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiPathItem {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub summary: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub get: Option<MurApiOperation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub post: Option<MurApiOperation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub put: Option<MurApiOperation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub delete: Option<MurApiOperation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub patch: Option<MurApiOperation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub head: Option<MurApiOperation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub options: Option<MurApiOperation>,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub parameters: Vec<MurApiParameter>,
}

impl MurApiPathItem {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(mut self, op: MurApiOperation) -> Self {
		self.get = Some(op);
		self
	}

	pub fn post(mut self, op: MurApiOperation) -> Self {
		self.post = Some(op);
		self
	}

	pub fn put(mut self, op: MurApiOperation) -> Self {
		self.put = Some(op);
		self
	}

	pub fn delete(mut self, op: MurApiOperation) -> Self {
		self.delete = Some(op);
		self
	}

	pub fn patch(mut self, op: MurApiOperation) -> Self {
		self.patch = Some(op);
		self
	}

	pub fn parameter(mut self, param: MurApiParameter) -> Self {
		self.parameters.push(param);
		self
	}
}
