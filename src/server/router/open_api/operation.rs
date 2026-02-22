use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
	external_doc::MurApiExternalDocs, parameter::MurApiParameter, request_body::MurApiRequestBody,
	response::MurApiResponse, schema::MurApiSchema,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiOperation {
	#[serde(rename = "operationId", skip_serializing_if = "Option::is_none")]
	pub operation_id: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub summary: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub tags: Vec<String>,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub parameters: Vec<MurApiParameter>,
	#[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
	pub request_body: Option<MurApiRequestBody>,
	pub responses: IndexMap<String, MurApiResponse>,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub security: Vec<HashMap<String, Vec<String>>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub deprecated: Option<bool>,
	#[serde(rename = "externalDocs", skip_serializing_if = "Option::is_none")]
	pub external_docs: Option<MurApiExternalDocs>,
}

impl MurApiOperation {
	pub fn new(summary: impl Into<String>) -> Self {
		let mut responses = IndexMap::new();
		responses.insert(
			"200".to_string(),
			MurApiResponse::new("Successful response"),
		);
		Self {
			summary: Some(summary.into()),
			responses,
			..Default::default()
		}
	}

	pub fn operation_id(mut self, id: impl Into<String>) -> Self {
		self.operation_id = Some(id.into());
		self
	}

	pub fn description(mut self, desc: impl Into<String>) -> Self {
		self.description = Some(desc.into());
		self
	}

	pub fn tag(mut self, tag: impl Into<String>) -> Self {
		self.tags.push(tag.into());
		self
	}

	pub fn tags<I, S>(mut self, tags: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		self.tags.extend(tags.into_iter().map(|s| s.into()));
		self
	}

	pub fn parameter(mut self, param: MurApiParameter) -> Self {
		self.parameters.push(param);
		self
	}

	pub fn path_param(self, name: impl Into<String>, description: impl Into<String>) -> Self {
		self.parameter(MurApiParameter::path(name, description))
	}

	pub fn query_param(self, name: impl Into<String>, description: impl Into<String>) -> Self {
		self.parameter(MurApiParameter::query(name, description))
	}

	pub fn header_param(self, name: impl Into<String>, description: impl Into<String>) -> Self {
		self.parameter(MurApiParameter::header(name, description))
	}

	pub fn request_body(mut self, body: MurApiRequestBody) -> Self {
		self.request_body = Some(body);
		self
	}

	pub fn json_body(self, schema: MurApiSchema) -> Self {
		self.request_body(MurApiRequestBody::json(schema))
	}

	pub fn json_body_with_desc(self, schema: MurApiSchema, description: impl Into<String>) -> Self {
		self.request_body(MurApiRequestBody::json(schema).description(description))
	}

	pub fn response(mut self, status: u16, description: impl Into<String>) -> Self {
		self.responses
			.insert(status.to_string(), MurApiResponse::new(description));
		self
	}

	pub fn response_with_schema(
		mut self,
		status: u16,
		description: impl Into<String>,
		schema: MurApiSchema,
	) -> Self {
		self.responses.insert(
			status.to_string(),
			MurApiResponse::new(description).json(schema),
		);
		self
	}

	pub fn security(mut self, name: impl Into<String>, scopes: Vec<String>) -> Self {
		let mut req = HashMap::new();
		req.insert(name.into(), scopes);
		self.security.push(req);
		self
	}

	pub fn bearer_auth(self) -> Self {
		self.security("bearerAuth", vec![])
	}

	pub fn api_key_auth(self) -> Self {
		self.security("apiKey", vec![])
	}

	pub fn deprecated(mut self) -> Self {
		self.deprecated = Some(true);
		self
	}
}
