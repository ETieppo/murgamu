use super::AllowedHeaders;
use super::AllowedOrigins;
use http::Method;
use std::{borrow::Cow, collections::HashSet};

pub static DEFAULT_METHODS: &[Method] = &[
	Method::GET,
	Method::POST,
	Method::PUT,
	Method::DELETE,
	Method::PATCH,
	Method::HEAD,
	Method::OPTIONS,
];

#[derive(Debug, Clone)]
pub struct MurCorsConfig {
	pub allowed_origins: AllowedOrigins,
	pub allowed_methods: Cow<'static, [Method]>,
	pub allowed_headers: AllowedHeaders,
	pub exposed_headers: HashSet<String>,
	pub allow_credentials: bool,
	pub max_age: Option<u64>,
	pub allow_private_network: bool,
	pub send_vary: bool,
}

impl Default for MurCorsConfig {
	fn default() -> Self {
		Self {
			allowed_origins: AllowedOrigins::Any,
			allowed_methods: Cow::Borrowed(DEFAULT_METHODS),
			allowed_headers: AllowedHeaders::Any,
			exposed_headers: HashSet::new(),
			allow_credentials: false,
			max_age: Some(86400), // 24 hours
			allow_private_network: false,
			send_vary: true,
		}
	}
}
