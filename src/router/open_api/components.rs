use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::{
	parameter::MurApiParameter, request_body::MurApiRequestBody, response::MurApiResponse,
	schema::MurApiSchema, security_scheme::MurApiSecurityScheme,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiComponents {
	#[serde(skip_serializing_if = "IndexMap::is_empty", default)]
	pub schemas: IndexMap<String, MurApiSchema>,
	#[serde(skip_serializing_if = "IndexMap::is_empty", default)]
	pub responses: IndexMap<String, MurApiResponse>,
	#[serde(skip_serializing_if = "IndexMap::is_empty", default)]
	pub parameters: IndexMap<String, MurApiParameter>,
	#[serde(
		rename = "requestBodies",
		skip_serializing_if = "IndexMap::is_empty",
		default
	)]
	pub request_bodies: IndexMap<String, MurApiRequestBody>,
	#[serde(
		rename = "securitySchemes",
		skip_serializing_if = "IndexMap::is_empty",
		default
	)]
	pub security_schemes: IndexMap<String, MurApiSecurityScheme>,
}
