use http::StatusCode;
use hyper::body::Bytes;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MurTestResponse {
	pub status: StatusCode,
	pub headers: HashMap<String, String>,
	pub body: Bytes,
}

impl MurTestResponse {
	pub fn new(status: StatusCode, headers: HashMap<String, String>, body: Bytes) -> Self {
		Self {
			status,
			headers,
			body,
		}
	}

	pub fn status(&self) -> StatusCode {
		self.status
	}

	pub fn headers(&self) -> &HashMap<String, String> {
		&self.headers
	}

	pub fn header(&self, name: &str) -> Option<&String> {
		let name_lower = name.to_lowercase();
		self.headers
			.iter()
			.find(|(k, _)| k.to_lowercase() == name_lower)
			.map(|(_, v)| v)
	}

	pub fn body(&self) -> &Bytes {
		&self.body
	}

	pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
		String::from_utf8(self.body.to_vec())
	}

	pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
		serde_json::from_slice(&self.body)
	}

	pub fn json_value(&self) -> Result<Value, serde_json::Error> {
		serde_json::from_slice(&self.body)
	}
}
