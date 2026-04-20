use crate::server::aliases::MurRes;
use http::StatusCode;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Response;
use serde::Serialize;
use std::collections::HashMap;

/// A fluent builder for constructing HTTP responses.
///
/// `MurResponseBuilder` lets you set a status code, add arbitrary headers,
/// and then finalise the response with one of the body methods. It is
/// obtained from [`MurHttpResponse`](crate::MurHttpResponse) factory methods
/// or constructed directly.
///
/// # Example
///
/// ```rust,ignore
/// MurResponseBuilder::new()
///     .status(StatusCode::CREATED)
///     .header("X-Request-Id", "abc123")
///     .json(serde_json::json!({ "id": 1 }))
/// ```
#[derive(Debug, Clone)]
pub struct MurResponseBuilder {
	status: StatusCode,
	headers: HashMap<String, String>,
}

impl MurResponseBuilder {
	/// Creates a new builder with `200 OK` and no headers.
	pub fn new() -> Self {
		Self {
			status: StatusCode::OK,
			headers: HashMap::new(),
		}
	}

	/// Sets the HTTP status code.
	pub fn status(mut self, status: StatusCode) -> Self {
		self.status = status;
		self
	}

	/// Appends a single response header.
	pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
		self.headers.insert(name.into(), value.into());
		self
	}

	/// Appends multiple response headers from an iterator of `(name, value)` pairs.
	pub fn headers(mut self, headers: impl IntoIterator<Item = (String, String)>) -> Self {
		self.headers.extend(headers);
		self
	}

	/// Sets the `Content-Type` header.
	pub fn content_type(self, content_type: impl Into<String>) -> Self {
		self.header("Content-Type", content_type)
	}

	/// Serializes `body` as JSON and finalises the response.
	///
	/// Automatically sets `Content-Type: application/json` unless a
	/// `Content-Type` header was already added.
	pub fn json<T: Serialize>(self, body: T) -> MurRes {
		let json_body = serde_json::to_string(&body)
			.unwrap_or_else(|e| format!(r#"{{"error":"Serialization failed: {}"}}"#, e));
		let mut builder = Response::builder().status(self.status);

		for (name, value) in &self.headers {
			builder = builder.header(name.as_str(), value.as_str());
		}

		if !self.headers.contains_key("Content-Type") {
			builder = builder.header("Content-Type", "application/json");
		}

		Ok(builder.body(Full::new(Bytes::from(json_body))).unwrap())
	}

	/// Writes `body` as plain text and finalises the response.
	///
	/// Automatically sets `Content-Type: text/plain; charset=utf-8`.
	pub fn text(self, body: impl Into<String>) -> MurRes {
		let text_body = body.into();
		let mut builder = Response::builder().status(self.status);

		for (name, value) in &self.headers {
			builder = builder.header(name.as_str(), value.as_str());
		}

		if !self.headers.contains_key("Content-Type") {
			builder = builder.header("Content-Type", "text/plain; charset=utf-8");
		}

		Ok(builder.body(Full::new(Bytes::from(text_body))).unwrap())
	}

	/// Writes `body` as HTML and finalises the response.
	///
	/// Automatically sets `Content-Type: text/html; charset=utf-8`.
	pub fn html(self, body: impl Into<String>) -> MurRes {
		let html_body = body.into();
		let mut builder = Response::builder().status(self.status);

		for (name, value) in &self.headers {
			builder = builder.header(name.as_str(), value.as_str());
		}

		if !self.headers.contains_key("Content-Type") {
			builder = builder.header("Content-Type", "text/html; charset=utf-8");
		}

		Ok(builder.body(Full::new(Bytes::from(html_body))).unwrap())
	}

	/// Writes raw bytes as the response body.
	///
	/// Automatically sets `Content-Type: application/octet-stream` unless
	/// overridden.
	pub fn bytes(self, body: impl Into<Bytes>) -> MurRes {
		let mut builder = Response::builder().status(self.status);

		for (name, value) in &self.headers {
			builder = builder.header(name.as_str(), value.as_str());
		}

		if !self.headers.contains_key("Content-Type") {
			builder = builder.header("Content-Type", "application/octet-stream");
		}

		Ok(builder.body(Full::new(body.into())).unwrap())
	}

	/// Finalises the response with an empty body.
	pub fn empty(self) -> MurRes {
		let mut builder = Response::builder().status(self.status);

		for (name, value) in &self.headers {
			builder = builder.header(name.as_str(), value.as_str());
		}

		Ok(builder.body(Full::new(Bytes::new())).unwrap())
	}

	/// Sets the `Location` header and finalises with an empty body.
	///
	/// Intended for redirect responses (`3xx`).
	pub fn redirect(self, location: impl Into<String>) -> MurRes {
		self.header("Location", location).empty()
	}
}

impl Default for MurResponseBuilder {
	fn default() -> Self {
		Self::new()
	}
}
