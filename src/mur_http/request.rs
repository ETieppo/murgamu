use crate::container::core::MurServiceContainer;
use crate::core::error::MurError;
use crate::router::entry::MurRouteAccessControl;
use crate::traits::MurService;
use crate::utils::mur_codec::MurCodec;
use http::request::Parts;
use hyper::body::Bytes;
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::OnceLock;

pub type MurReq = MurRequestContext;

#[derive(Clone)]
pub struct MurRequestContext {
	pub parts: Parts,
	pub body: Option<Bytes>,
	pub path_params: HashMap<String, String>,
	pub container: Arc<MurServiceContainer>,
	query_cache: OnceLock<HashMap<String, String>>,
	pub(crate) access_control: Option<MurRouteAccessControl>,
}

impl MurRequestContext {
	pub fn new(
		parts: Parts,
		body: Option<Bytes>,
		path_params: HashMap<String, String>,
		container: Arc<MurServiceContainer>,
	) -> Self {
		Self {
			parts,
			body,
			path_params,
			container,
			query_cache: OnceLock::new(),
			access_control: None,
		}
	}

	pub fn service<T: MurService>(&self) -> Option<Arc<T>> {
		self.container.get::<T>()
	}

	pub fn service_required<T: MurService>(&self) -> Arc<T> {
		self.container.get_required::<T>()
	}

	pub fn path_param(&self, name: &str) -> Option<&str> {
		self.path_params.get(name).map(|s| s.as_str())
	}

	pub fn param_or<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
		self.path_param(name).unwrap_or(default)
	}

	pub fn param_as<T: std::str::FromStr>(&self, name: &str) -> Option<T> {
		self.path_param(name).and_then(|s| s.parse().ok())
	}

	pub fn params(&self) -> &HashMap<String, String> {
		&self.path_params
	}

	pub fn has_param(&self, name: &str) -> bool {
		self.path_params.contains_key(name)
	}

	pub fn path(&self) -> &str {
		self.parts.uri.path()
	}

	pub fn path_segment(&self, index: usize) -> Option<&str> {
		self.path().trim_start_matches('/').split('/').nth(index)
	}

	pub fn path_segments(&self) -> Vec<&str> {
		self.path()
			.trim_start_matches('/')
			.split('/')
			.filter(|s| !s.is_empty())
			.collect()
	}

	pub fn query_map(&self) -> &HashMap<String, String> {
		self.query_cache.get_or_init(|| {
			self.parts
				.uri
				.query()
				.and_then(|q| serde_urlencoded::from_str(q).ok())
				.unwrap_or_default()
		})
	}

	pub fn query_param(&self, name: &str) -> Option<&str> {
		self.query_map().get(name).map(|s| s.as_str())
	}

	pub fn query_param_or<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
		self.query_param(name).unwrap_or(default)
	}

	pub fn query_param_as<T: std::str::FromStr>(&self, name: &str) -> Option<T> {
		self.query_param(name).and_then(|s| s.parse().ok())
	}

	pub fn has_query_param(&self, name: &str) -> bool {
		self.query_param(name).is_some()
	}

	pub fn query_string(&self) -> Option<&str> {
		self.parts.uri.query()
	}

	pub fn header(&self, name: &str) -> Option<&str> {
		self.parts.headers.get(name).and_then(|v| v.to_str().ok())
	}

	pub fn header_or<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
		self.header(name).unwrap_or(default)
	}

	pub fn has_header(&self, name: &str) -> bool {
		self.parts.headers.contains_key(name)
	}

	pub fn content_type(&self) -> Option<&str> {
		self.header("Content-Type")
	}

	pub fn is_json(&self) -> bool {
		self.content_type()
			.map(|ct| ct.contains("application/json"))
			.unwrap_or(false)
	}

	pub fn is_form(&self) -> bool {
		self.content_type()
			.map(|ct| ct.contains("application/x-www-form-urlencoded"))
			.unwrap_or(false)
	}

	pub fn is_multipart(&self) -> bool {
		self.content_type()
			.map(|ct| ct.contains("multipart/form-data"))
			.unwrap_or(false)
	}

	pub fn authorization(&self) -> Option<&str> {
		self.header("Authorization")
	}

	pub fn bearer_token(&self) -> Option<&str> {
		self.authorization()
			.and_then(|auth| auth.strip_prefix("Bearer "))
	}

	pub fn basic_auth(&self) -> Option<(String, String)> {
		self.authorization()
			.and_then(|auth| auth.strip_prefix("Basic "))
			.and_then(|encoded| {
				let decoded = MurCodec::base64_decode(encoded).ok()?;
				let decoded_str = String::from_utf8(decoded).ok()?;
				let mut parts = decoded_str.splitn(2, ':');
				let username = parts.next()?.to_string();
				let password = parts.next()?.to_string();
				Some((username, password))
			})
	}

	pub fn user_agent(&self) -> Option<&str> {
		self.header("User-Agent")
	}

	pub fn accept(&self) -> Option<&str> {
		self.header("Accept")
	}

	pub fn accepts_json(&self) -> bool {
		self.accept()
			.map(|a| a.contains("application/json") || a.contains("*/*"))
			.unwrap_or(true)
	}

	pub fn method(&self) -> &http::Method {
		&self.parts.method
	}

	pub fn uri(&self) -> &http::Uri {
		&self.parts.uri
	}

	pub fn is_get(&self) -> bool {
		self.parts.method == http::Method::GET
	}

	pub fn is_post(&self) -> bool {
		self.parts.method == http::Method::POST
	}

	pub fn is_put(&self) -> bool {
		self.parts.method == http::Method::PUT
	}

	pub fn is_delete(&self) -> bool {
		self.parts.method == http::Method::DELETE
	}

	pub fn is_patch(&self) -> bool {
		self.parts.method == http::Method::PATCH
	}

	pub fn json<T: DeserializeOwned>(&self) -> Result<T, MurError> {
		let body = self
			.body
			.as_ref()
			.ok_or_else(|| MurError::BadRequest("Missing request body".to_string()))?;

		serde_json::from_slice(body)
			.map_err(|e| MurError::BadRequest(format!("Invalid JSON: {}", e)))
	}

	pub fn body_bytes(&self) -> Option<&Bytes> {
		self.body.as_ref()
	}

	pub fn body_string(&self) -> Result<String, MurError> {
		let body = self
			.body
			.as_ref()
			.ok_or_else(|| MurError::BadRequest("Missing request body".to_string()))?;

		String::from_utf8(body.to_vec())
			.map_err(|e| MurError::BadRequest(format!("Invalid UTF-8 in body: {}", e)))
	}

	pub fn form<T: DeserializeOwned>(&self) -> Result<T, MurError> {
		let body = self.body_string()?;
		serde_urlencoded::from_str(&body)
			.map_err(|e| MurError::BadRequest(format!("Invalid form data: {}", e)))
	}

	pub fn has_body(&self) -> bool {
		self.body.is_some()
	}

	pub fn content_length(&self) -> Option<usize> {
		self.header("Content-Length").and_then(|s| s.parse().ok())
	}

	pub fn client_ip(&self) -> Option<&str> {
		self.header("X-Forwarded-For")
			.and_then(|s| s.split(',').next())
			.map(|s| s.trim())
			.or_else(|| self.header("X-Real-IP"))
	}

	pub fn host(&self) -> Option<&str> {
		self.header("Host")
	}

	pub fn origin(&self) -> Option<&str> {
		self.header("Origin")
	}

	pub fn referer(&self) -> Option<&str> {
		self.header("Referer")
	}

	pub fn header_all(&self, name: &str) -> Vec<&str> {
		self.parts
			.headers
			.get_all(name)
			.iter()
			.filter_map(|v| v.to_str().ok())
			.collect()
	}

	pub fn typed_query<T: DeserializeOwned>(&self) -> Result<T, MurError> {
		let query = self.parts.uri.query().unwrap_or("");
		serde_urlencoded::from_str(query)
			.map_err(|e| MurError::BadRequest(format!("Failed to parse query params: {}", e)))
	}

	pub fn with_access_control(mut self, access_control: MurRouteAccessControl) -> Self {
		self.access_control = Some(access_control);
		self
	}

	pub fn is_public_route(&self) -> bool {
		self.access_control
			.as_ref()
			.map(|ac| ac.is_public)
			.unwrap_or(false)
	}

	pub fn allowed_roles(&self) -> Option<&HashSet<String>> {
		self.access_control.as_ref().map(|ac| &ac.allowed_roles)
	}

	pub fn has_allowed_role(&self, role: &str) -> bool {
		self.access_control
			.as_ref()
			.map(|ac| ac.allowed_roles.is_empty() || ac.allowed_roles.contains(role))
			.unwrap_or(true)
	}
}

impl std::fmt::Debug for MurRequestContext {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurRequestContext")
			.field("method", &self.parts.method)
			.field("uri", &self.parts.uri)
			.field("path_params", &self.path_params)
			.field("has_body", &self.body.is_some())
			.finish()
	}
}
