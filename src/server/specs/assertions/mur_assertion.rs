use super::MurTestResponse;
use super::get_json_path;
use http::StatusCode;
use serde_json::Value;
use std::fmt;

pub trait MurResponseAssertions {
	fn assert_status(&self, expected: StatusCode) -> &Self;
	fn assert_success(&self) -> &Self;
	fn assert_client_error(&self) -> &Self;
	fn assert_server_error(&self) -> &Self;
	fn assert_header_exists(&self, name: &str) -> &Self;
	fn assert_header(&self, name: &str, expected: &str) -> &Self;
	fn assert_header_contains(&self, name: &str, substring: &str) -> &Self;
	fn assert_body(&self, expected: &str) -> &Self;
	fn assert_body_contains(&self, substring: &str) -> &Self;
	fn assert_body_matches(&self, pattern: &str) -> &Self;
	fn assert_body_empty(&self) -> &Self;
	fn assert_json_valid(&self) -> &Self;
	fn assert_json_has(&self, key: &str) -> &Self;
	fn assert_json_eq<T: serde::Serialize + PartialEq + fmt::Debug>(
		&self,
		key: &str,
		expected: T,
	) -> &Self;
	fn assert_json_contains(&self, key: &str, expected: &str) -> &Self;
	fn assert_json_array_len(&self, len: usize) -> &Self;
	fn assert_json_path(&self, path: &str, expected: &str) -> &Self;
	fn assert_content_type_json(&self) -> &Self;
	fn assert_content_type_text(&self) -> &Self;
	fn assert_content_type_html(&self) -> &Self;
}

impl MurResponseAssertions for MurTestResponse {
	fn assert_status(&self, expected: StatusCode) -> &Self {
		assert_eq!(
			self.status, expected,
			"Status code mismatch: expected {}, got {}",
			expected, self.status
		);
		self
	}

	fn assert_success(&self) -> &Self {
		assert!(
			self.status.is_success(),
			"Expected success status (2xx), got {}",
			self.status
		);
		self
	}

	fn assert_client_error(&self) -> &Self {
		assert!(
			self.status.is_client_error(),
			"Expected client error status (4xx), got {}",
			self.status
		);
		self
	}

	fn assert_server_error(&self) -> &Self {
		assert!(
			self.status.is_server_error(),
			"Expected server error status (5xx), got {}",
			self.status
		);
		self
	}

	fn assert_header_exists(&self, name: &str) -> &Self {
		assert!(
			self.header(name).is_some(),
			"Expected header '{}' to exist",
			name
		);
		self
	}

	fn assert_header(&self, name: &str, expected: &str) -> &Self {
		match self.header(name) {
			Some(value) => {
				assert_eq!(
					value, expected,
					"Header '{}' mismatch: expected '{}', got '{}'",
					name, expected, value
				);
			}
			None => {
				panic!("Header '{}' not found", name);
			}
		}
		self
	}

	fn assert_header_contains(&self, name: &str, substring: &str) -> &Self {
		match self.header(name) {
			Some(value) => {
				assert!(
					value.contains(substring),
					"Header '{}' does not contain '{}', actual value: '{}'",
					name,
					substring,
					value
				);
			}
			None => {
				panic!("Header '{}' not found", name);
			}
		}
		self
	}

	fn assert_body(&self, expected: &str) -> &Self {
		let body = self.text().unwrap_or_default();
		assert_eq!(
			body, expected,
			"Body mismatch:\n  Expected: {}\n  Actual: {}",
			expected, body
		);
		self
	}

	fn assert_body_contains(&self, substring: &str) -> &Self {
		let body = self.text().unwrap_or_default();
		assert!(
			body.contains(substring),
			"Body does not contain '{}', actual body:\n{}",
			substring,
			body
		);
		self
	}

	fn assert_body_matches(&self, pattern: &str) -> &Self {
		let body = self.text().unwrap_or_default();
		let regex = regex::Regex::new(pattern).expect("Invalid regex pattern");
		assert!(
			regex.is_match(&body),
			"Body does not match pattern '{}', actual body:\n{}",
			pattern,
			body
		);
		self
	}

	fn assert_body_empty(&self) -> &Self {
		assert!(
			self.body.is_empty(),
			"Expected empty body, got {} bytes",
			self.body.len()
		);
		self
	}

	fn assert_json_valid(&self) -> &Self {
		self.json_value().expect("Body is not valid JSON");
		self
	}

	fn assert_json_has(&self, key: &str) -> &Self {
		let json: Value = self.json_value().expect("Body is not valid JSON");
		assert!(
			json.get(key).is_some(),
			"JSON does not contain key '{}', actual JSON:\n{}",
			key,
			serde_json::to_string_pretty(&json).unwrap_or_default()
		);
		self
	}

	fn assert_json_eq<T: serde::Serialize + PartialEq + fmt::Debug>(
		&self,
		key: &str,
		expected: T,
	) -> &Self {
		let json: Value = self.json_value().expect("Body is not valid JSON");
		let expected_value =
			serde_json::to_value(&expected).expect("Failed to serialize expected value");

		match json.get(key) {
			Some(actual) => {
				assert_eq!(
					actual, &expected_value,
					"JSON field '{}' mismatch:\n  Expected: {:?}\n  Actual: {}",
					key, expected, actual
				);
			}
			None => {
				panic!(
					"JSON does not contain key '{}', actual JSON:\n{}",
					key,
					serde_json::to_string_pretty(&json).unwrap_or_default()
				);
			}
		}
		self
	}

	fn assert_json_contains(&self, key: &str, expected: &str) -> &Self {
		let json: Value = self.json_value().expect("Body is not valid JSON");
		match json.get(key) {
			Some(value) => {
				let value_str = match value {
					Value::String(s) => s.clone(),
					_ => value.to_string(),
				};
				assert!(
					value_str.contains(expected),
					"JSON field '{}' does not contain '{}', actual value: '{}'",
					key,
					expected,
					value_str
				);
			}
			None => {
				panic!(
					"JSON does not contain key '{}', actual JSON:\n{}",
					key,
					serde_json::to_string_pretty(&json).unwrap_or_default()
				);
			}
		}
		self
	}

	fn assert_json_array_len(&self, len: usize) -> &Self {
		let json: Value = self.json_value().expect("Body is not valid JSON");
		match json.as_array() {
			Some(arr) => {
				assert_eq!(
					arr.len(),
					len,
					"JSON array length mismatch: expected {}, got {}",
					len,
					arr.len()
				);
			}
			None => {
				panic!("JSON is not an array");
			}
		}
		self
	}

	fn assert_json_path(&self, path: &str, expected: &str) -> &Self {
		let json: Value = self.json_value().expect("Body is not valid JSON");
		let value = get_json_path(&json, path);

		match value {
			Some(v) => {
				let value_str = match v {
					Value::String(s) => s.clone(),
					_ => v.to_string(),
				};
				assert_eq!(
					value_str, expected,
					"JSON path '{}' mismatch:\n  Expected: {}\n  Actual: {}",
					path, expected, value_str
				);
			}
			None => {
				panic!(
					"JSON path '{}' not found, actual JSON:\n{}",
					path,
					serde_json::to_string_pretty(&json).unwrap_or_default()
				);
			}
		}
		self
	}

	fn assert_content_type_json(&self) -> &Self {
		self.assert_header_contains("content-type", "application/json")
	}

	fn assert_content_type_text(&self) -> &Self {
		self.assert_header_contains("content-type", "text/plain")
	}

	fn assert_content_type_html(&self) -> &Self {
		self.assert_header_contains("content-type", "text/html")
	}
}
