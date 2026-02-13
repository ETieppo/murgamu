use crate::types::MurRes;
use http::StatusCode;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Response;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MurResponseBuilder {
	status: StatusCode,
	headers: HashMap<String, String>,
}

impl MurResponseBuilder {
	pub fn new() -> Self {
		Self {
			status: StatusCode::OK,
			headers: HashMap::new(),
		}
	}

	pub fn status(mut self, status: StatusCode) -> Self {
		self.status = status;
		self
	}

	pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
		self.headers.insert(name.into(), value.into());
		self
	}

	pub fn headers(mut self, headers: impl IntoIterator<Item = (String, String)>) -> Self {
		self.headers.extend(headers);
		self
	}

	pub fn content_type(self, content_type: impl Into<String>) -> Self {
		self.header("Content-Type", content_type)
	}

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

	pub fn empty(self) -> MurRes {
		let mut builder = Response::builder().status(self.status);

		for (name, value) in &self.headers {
			builder = builder.header(name.as_str(), value.as_str());
		}

		Ok(builder.body(Full::new(Bytes::new())).unwrap())
	}

	pub fn redirect(self, location: impl Into<String>) -> MurRes {
		self.header("Location", location).empty()
	}
}

impl Default for MurResponseBuilder {
	fn default() -> Self {
		Self::new()
	}
}
