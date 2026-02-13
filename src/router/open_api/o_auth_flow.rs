use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiOAuthFlows {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub implicit: Option<MurApiOAuthFlow>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub password: Option<MurApiOAuthFlow>,
	#[serde(rename = "clientCredentials", skip_serializing_if = "Option::is_none")]
	pub client_credentials: Option<MurApiOAuthFlow>,
	#[serde(rename = "authorizationCode", skip_serializing_if = "Option::is_none")]
	pub authorization_code: Option<MurApiOAuthFlow>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiOAuthFlow {
	#[serde(rename = "authorizationUrl", skip_serializing_if = "Option::is_none")]
	pub authorization_url: Option<String>,
	#[serde(rename = "tokenUrl", skip_serializing_if = "Option::is_none")]
	pub token_url: Option<String>,
	#[serde(rename = "refreshUrl", skip_serializing_if = "Option::is_none")]
	pub refresh_url: Option<String>,
	pub scopes: HashMap<String, String>,
}
