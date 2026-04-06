use super::o_auth_flow::MurApiOAuthFlows;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MurApiSecurityScheme {
	ApiKey {
		name: String,
		#[serde(rename = "in")]
		location: String,
		#[serde(skip_serializing_if = "Option::is_none")]
		description: Option<String>,
	},
	Http {
		scheme: String,
		#[serde(rename = "bearerFormat", skip_serializing_if = "Option::is_none")]
		bearer_format: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		description: Option<String>,
	},
	Oauth2 {
		flows: Box<MurApiOAuthFlows>,
		#[serde(skip_serializing_if = "Option::is_none")]
		description: Option<String>,
	},
	OpenIdConnect {
		#[serde(rename = "openIdConnectUrl")]
		open_id_connect_url: String,
		#[serde(skip_serializing_if = "Option::is_none")]
		description: Option<String>,
	},
}

impl MurApiSecurityScheme {
	pub fn bearer() -> Self {
		MurApiSecurityScheme::Http {
			scheme: "bearer".to_string(),
			bearer_format: Some("JWT".to_string()),
			description: Some("JWT Bearer token authentication".to_string()),
		}
	}

	pub fn basic() -> Self {
		MurApiSecurityScheme::Http {
			scheme: "basic".to_string(),
			bearer_format: None,
			description: Some("HTTP Basic authentication".to_string()),
		}
	}

	pub fn api_key_header(name: impl Into<String>) -> Self {
		MurApiSecurityScheme::ApiKey {
			name: name.into(),
			location: "header".to_string(),
			description: Some("API Key authentication via header".to_string()),
		}
	}

	pub fn api_key_query(name: impl Into<String>) -> Self {
		MurApiSecurityScheme::ApiKey {
			name: name.into(),
			location: "query".to_string(),
			description: Some("API Key authentication via query parameter".to_string()),
		}
	}
}
