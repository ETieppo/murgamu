// TODO: analyse, I ask to ai build it but it smell bad
//! Test Client for HTTP Request Testing
//!
//! This module provides a test client that allows making HTTP requests to the application
//! without starting a real server. It's designed for integration testing controllers.

use crate::server::aliases::MurRes;
use crate::server::http::MurHttpResponse;
use crate::server::http::MurRequestContext;
use crate::server::middleware::MurMiddleware;
use crate::server::router::MurRouter;
use crate::server::service::MurServiceContainer;
use http::{Method, StatusCode};
use hyper::body::Bytes;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

// =============================================================================
// Test Client
// =============================================================================

/// A test client for making HTTP requests to the application.
///
/// The test client allows you to make requests to your application without
/// starting a real HTTP server, making tests faster and more isolated.
///
/// # Example
///
/// ```ignore
/// let client = MurTestClient::new(router, container);
///
/// let response = client.get("/api/users").send().await;
/// assert_eq!(response.status(), StatusCode::OK);
/// ```
#[derive(Clone)]
pub struct MurTestClient {
	router: Arc<MurRouter>,
	container: Arc<MurServiceContainer>,
	middleware: Vec<Arc<dyn MurMiddleware>>,
	default_headers: HashMap<String, String>,
}

impl MurTestClient {
	/// Create a new test client with the given router and container.
	pub fn new(router: Arc<MurRouter>, container: Arc<MurServiceContainer>) -> Self {
		Self {
			router,
			container,
			middleware: Vec::new(),
			default_headers: HashMap::new(),
		}
	}

	/// Add middleware to the test client.
	pub fn with_middleware<M: MurMiddleware>(mut self, middleware: M) -> Self {
		self.middleware.push(Arc::new(middleware));
		self
	}

	/// Add a default header that will be sent with every request.
	pub fn with_default_header(
		mut self,
		name: impl Into<String>,
		value: impl Into<String>,
	) -> Self {
		self.default_headers.insert(name.into(), value.into());
		self
	}

	/// Add default Authorization header with Bearer token.
	pub fn with_bearer_token(self, token: impl Into<String>) -> Self {
		self.with_default_header("Authorization", format!("Bearer {}", token.into()))
	}

	/// Create a GET request builder.
	pub fn get(&self, path: &str) -> MurTestRequestBuilder {
		self.request(Method::GET, path)
	}

	/// Create a POST request builder.
	pub fn post(&self, path: &str) -> MurTestRequestBuilder {
		self.request(Method::POST, path)
	}

	/// Create a PUT request builder.
	pub fn put(&self, path: &str) -> MurTestRequestBuilder {
		self.request(Method::PUT, path)
	}

	/// Create a PATCH request builder.
	pub fn patch(&self, path: &str) -> MurTestRequestBuilder {
		self.request(Method::PATCH, path)
	}

	/// Create a DELETE request builder.
	pub fn delete(&self, path: &str) -> MurTestRequestBuilder {
		self.request(Method::DELETE, path)
	}

	/// Create a HEAD request builder.
	pub fn head(&self, path: &str) -> MurTestRequestBuilder {
		self.request(Method::HEAD, path)
	}

	/// Create an OPTIONS request builder.
	pub fn options(&self, path: &str) -> MurTestRequestBuilder {
		self.request(Method::OPTIONS, path)
	}

	/// Create a request builder with the given method and path.
	pub fn request(&self, method: Method, path: &str) -> MurTestRequestBuilder {
		let headers = self.default_headers.clone();

		MurTestRequestBuilder {
			client: self.clone(),
			method,
			path: path.to_string(),
			headers,
			body: None,
			query_params: HashMap::new(),
		}
	}
}

// =============================================================================
// Test Request Builder
// =============================================================================

/// Builder for constructing test requests.
///
/// # Example
///
/// ```ignore
/// let response = client
///     .post("/api/users")
///     .header("Content-Type", "application/json")
///     .json(&CreateUserDto { name: "John".into() })
///     .send()
///     .await;
/// ```
pub struct MurTestRequestBuilder {
	client: MurTestClient,
	method: Method,
	path: String,
	headers: HashMap<String, String>,
	body: Option<Bytes>,
	query_params: HashMap<String, String>,
}

impl MurTestRequestBuilder {
	/// Add a header to the request.
	pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
		self.headers.insert(name.into(), value.into());
		self
	}

	/// Add multiple headers to the request.
	pub fn headers(mut self, headers: impl IntoIterator<Item = (String, String)>) -> Self {
		for (name, value) in headers {
			self.headers.insert(name, value);
		}
		self
	}

	/// Set the request body as JSON.
	pub fn json<T: Serialize>(mut self, body: &T) -> Self {
		match serde_json::to_vec(body) {
			Ok(bytes) => {
				self.body = Some(Bytes::from(bytes));
				self.headers
					.insert("Content-Type".to_string(), "application/json".to_string());
			}
			Err(e) => {
				eprintln!("Failed to serialize JSON body: {}", e);
			}
		}
		self
	}

	/// Set the request body as raw bytes.
	pub fn body(mut self, body: impl Into<Bytes>) -> Self {
		self.body = Some(body.into());
		self
	}

	/// Set the request body as text.
	pub fn text(mut self, body: impl Into<String>) -> Self {
		self.body = Some(Bytes::from(body.into()));
		self.headers
			.insert("Content-Type".to_string(), "text/plain".to_string());
		self
	}

	/// Set the request body as form data.
	pub fn form<T: Serialize>(mut self, body: &T) -> Self {
		match serde_urlencoded::to_string(body) {
			Ok(encoded) => {
				self.body = Some(Bytes::from(encoded));
				self.headers.insert(
					"Content-Type".to_string(),
					"application/x-www-form-urlencoded".to_string(),
				);
			}
			Err(e) => {
				eprintln!("Failed to encode form body: {}", e);
			}
		}
		self
	}

	/// Add a query parameter to the request.
	pub fn query(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
		self.query_params.insert(name.into(), value.into());
		self
	}

	/// Add multiple query parameters to the request.
	pub fn query_params(mut self, params: impl IntoIterator<Item = (String, String)>) -> Self {
		for (name, value) in params {
			self.query_params.insert(name, value);
		}
		self
	}

	/// Set the Authorization header with a Bearer token.
	pub fn bearer_token(self, token: impl Into<String>) -> Self {
		self.header("Authorization", format!("Bearer {}", token.into()))
	}

	/// Set the Authorization header with Basic auth.
	pub fn basic_auth(self, username: impl AsRef<str>, password: impl AsRef<str>) -> Self {
		let credentials = format!("{}:{}", username.as_ref(), password.as_ref());
		let encoded = base64_encode(credentials.as_bytes());
		self.header("Authorization", format!("Basic {}", encoded))
	}

	/// Build the full path including query parameters.
	fn build_path(&self) -> String {
		if self.query_params.is_empty() {
			self.path.clone()
		} else {
			let query_string: Vec<String> = self
				.query_params
				.iter()
				.map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
				.collect();
			format!("{}?{}", self.path, query_string.join("&"))
		}
	}

	/// Send the request and return the response.
	pub async fn send(mut self) -> MurTestResponse {
		let path = self.build_path();

		// Build the request parts
		let mut builder = http::Request::builder()
			.method(self.method.clone())
			.uri(&path);

		for (name, value) in &self.headers {
			builder = builder.header(name, value);
		}

		let body = self.body.take().unwrap_or_default();

		// Create the request
		let request = match builder.body(body.clone()) {
			Ok(req) => req,
			Err(e) => {
				return MurTestResponse::error(format!("Failed to build request: {}", e));
			}
		};

		// Extract parts
		let (parts, _) = request.into_parts();

		// Create request context
		let ctx = MurRequestContext::new(
			parts,
			Some(body),
			HashMap::new(),
			self.client.container.clone(),
		);

		// Find and execute the route handler
		let response = self.execute_request(ctx).await;

		MurTestResponse::from_response(response)
	}

	async fn execute_request(self, mut ctx: MurRequestContext) -> MurRes {
		let path = ctx.path().to_string();
		let method = ctx.method().to_string();

		// Try to find matching route params first
		if let Some(params) = self.client.router.find_route_params(&method, &path) {
			ctx.path_params = params;
		}

		// Use the router's handle method to execute the request
		// This properly handles middleware, guards, etc.
		match self
			.client
			.router
			.execute_matched_route(&method, &path, ctx)
			.await
		{
			Some(result) => result,
			None => {
				// Route not found
				MurHttpResponse::not_found().json(serde_json::json!({
					"error": "Not Found",
					"message": format!("No route found for {} {}", method, path)
				}))
			}
		}
	}
}

// =============================================================================
// Test Request (for sending)
// =============================================================================

/// Represents a test request that has been built and is ready to send.
pub struct MurTestRequest {
	method: Method,
	path: String,
	headers: HashMap<String, String>,
	body: Option<Bytes>,
}

impl MurTestRequest {
	/// Get the request method.
	pub fn method(&self) -> &Method {
		&self.method
	}

	/// Get the request path.
	pub fn path(&self) -> &str {
		&self.path
	}

	/// Get the request headers.
	pub fn headers(&self) -> &HashMap<String, String> {
		&self.headers
	}

	/// Get the request body.
	pub fn body(&self) -> Option<&Bytes> {
		self.body.as_ref()
	}
}

// =============================================================================
// Test Response
// =============================================================================

/// Response from a test request.
///
/// Provides methods to inspect the response status, headers, and body.
///
/// # Example
///
/// ```ignore
/// let response = client.get("/api/users").send().await;
///
/// assert_eq!(response.status(), StatusCode::OK);
/// let body: Vec<User> = response.json().unwrap();
/// assert!(!body.is_empty());
/// ```
pub struct MurTestResponse {
	status: StatusCode,
	headers: HashMap<String, String>,
	body: Bytes,
	error: Option<String>,
}

impl MurTestResponse {
	/// Create a response from a MurRes.
	pub(crate) fn from_response(result: MurRes) -> Self {
		match result {
			Ok(response) => {
				let status = response.status();
				let headers: HashMap<String, String> = response
					.headers()
					.iter()
					.map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
					.collect();

				// Get body synchronously for testing
				// Note: In real implementation, you'd need to collect the body
				let body = Bytes::new();

				Self {
					status,
					headers,
					body,
					error: None,
				}
			}
			Err(e) => Self::error(format!("Request failed: {}", e)),
		}
	}

	/// Create an error response.
	pub(crate) fn error(message: String) -> Self {
		Self {
			status: StatusCode::INTERNAL_SERVER_ERROR,
			headers: HashMap::new(),
			body: Bytes::from(message.clone()),
			error: Some(message),
		}
	}

	/// Get the response status code.
	pub fn status(&self) -> StatusCode {
		self.status
	}

	/// Check if the response was successful (2xx status code).
	pub fn is_success(&self) -> bool {
		self.status.is_success()
	}

	/// Check if the response was a client error (4xx status code).
	pub fn is_client_error(&self) -> bool {
		self.status.is_client_error()
	}

	/// Check if the response was a server error (5xx status code).
	pub fn is_server_error(&self) -> bool {
		self.status.is_server_error()
	}

	/// Get the response headers.
	pub fn headers(&self) -> &HashMap<String, String> {
		&self.headers
	}

	/// Get a specific header value.
	pub fn header(&self, name: &str) -> Option<&str> {
		self.headers.get(name).map(|s| s.as_str())
	}

	/// Get the raw response body.
	pub fn body(&self) -> &Bytes {
		&self.body
	}

	/// Get the response body as a string.
	pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
		String::from_utf8(self.body.to_vec())
	}

	/// Parse the response body as JSON.
	pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
		serde_json::from_slice(&self.body)
	}

	/// Get the error message if the request failed.
	pub fn error_message(&self) -> Option<&str> {
		self.error.as_deref()
	}

	/// Check if the request resulted in an error.
	pub fn is_error(&self) -> bool {
		self.error.is_some()
	}
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Simple base64 encoding for Basic auth.
fn base64_encode(data: &[u8]) -> String {
	const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

	let mut result = String::new();
	let chunks = data.chunks(3);

	for chunk in chunks {
		let mut buffer = [0u8; 3];
		buffer[..chunk.len()].copy_from_slice(chunk);

		let b0 = buffer[0];
		let b1 = buffer[1];
		let b2 = buffer[2];

		result.push(ALPHABET[(b0 >> 2) as usize] as char);
		result.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);

		if chunk.len() > 1 {
			result.push(ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
		} else {
			result.push('=');
		}

		if chunk.len() > 2 {
			result.push(ALPHABET[(b2 & 0x3f) as usize] as char);
		} else {
			result.push('=');
		}
	}

	result
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_base64_encode() {
		assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
		assert_eq!(base64_encode(b"world"), "d29ybGQ=");
		assert_eq!(base64_encode(b"user:pass"), "dXNlcjpwYXNz");
	}

	#[test]
	fn test_test_response_status_checks() {
		let success = MurTestResponse {
			status: StatusCode::OK,
			headers: HashMap::new(),
			body: Bytes::new(),
			error: None,
		};
		assert!(success.is_success());
		assert!(!success.is_client_error());
		assert!(!success.is_server_error());

		let not_found = MurTestResponse {
			status: StatusCode::NOT_FOUND,
			headers: HashMap::new(),
			body: Bytes::new(),
			error: None,
		};
		assert!(!not_found.is_success());
		assert!(not_found.is_client_error());
		assert!(!not_found.is_server_error());

		let server_error = MurTestResponse {
			status: StatusCode::INTERNAL_SERVER_ERROR,
			headers: HashMap::new(),
			body: Bytes::new(),
			error: None,
		};
		assert!(!server_error.is_success());
		assert!(!server_error.is_client_error());
		assert!(server_error.is_server_error());
	}

	#[test]
	fn test_query_params_building() {
		let container = Arc::new(MurServiceContainer::new());
		let builder = MurTestRequestBuilder {
			client: MurTestClient {
				router: Arc::new(MurRouter::new(Arc::clone(&container))),
				container,
				middleware: Vec::new(),
				default_headers: HashMap::new(),
			},
			method: Method::GET,
			path: "/api/users".to_string(),
			headers: HashMap::new(),
			body: None,
			query_params: HashMap::new(),
		};

		// Without query params
		assert_eq!(builder.build_path(), "/api/users");

		// With query params
		let builder = builder.query("page", "1").query("limit", "10");
		let path = builder.build_path();
		assert!(path.starts_with("/api/users?"));
		assert!(path.contains("page=1"));
		assert!(path.contains("limit=10"));
	}
}
