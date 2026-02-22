use http::StatusCode;
use hyper::body::Bytes;
use std::collections::HashMap;

use super::*;

#[test]
fn test_get_json_path_simple() {
	let json = serde_json::json!({
		"name": "John",
		"age": 30
	});

	assert_eq!(
		get_json_path(&json, "name"),
		Some(&serde_json::json!("John"))
	);
	assert_eq!(get_json_path(&json, "age"), Some(&serde_json::json!(30)));
	assert_eq!(get_json_path(&json, "missing"), None);
}

#[test]
fn test_get_json_path_nested() {
	let json = serde_json::json!({
		"user": {
			"name": "John",
			"address": {
				"city": "New York"
			}
		}
	});

	assert_eq!(
		get_json_path(&json, "user.name"),
		Some(&serde_json::json!("John"))
	);
	assert_eq!(
		get_json_path(&json, "user.address.city"),
		Some(&serde_json::json!("New York"))
	);
	assert_eq!(get_json_path(&json, "user.missing"), None);
}

#[test]
fn test_get_json_path_array() {
	let json = serde_json::json!({
		"users": [
			{"name": "John"},
			{"name": "Jane"}
		]
	});

	assert_eq!(
		get_json_path(&json, "users[0].name"),
		Some(&serde_json::json!("John"))
	);
	assert_eq!(
		get_json_path(&json, "users[1].name"),
		Some(&serde_json::json!("Jane"))
	);
	assert_eq!(get_json_path(&json, "users[2]"), None);
}

#[test]
fn test_assertion_error_display() {
	let error = AssertionError::new("status code", "200", "404");
	let display = format!("{}", error);

	assert!(display.contains("status code"));
	assert!(display.contains("200"));
	assert!(display.contains("404"));
}

#[test]
fn test_response_assertions_status() {
	let response = MurTestResponse::new(StatusCode::OK, HashMap::new(), Bytes::new());
	response.assert_status(StatusCode::OK);
	response.assert_success();
}

#[test]
fn test_response_assertions_headers() {
	let mut headers = HashMap::new();
	headers.insert("content-type".to_string(), "application/json".to_string());
	headers.insert("x-request-id".to_string(), "abc123".to_string());

	let response = MurTestResponse::new(StatusCode::OK, headers, Bytes::new());
	response
		.assert_header_exists("content-type")
		.assert_header("x-request-id", "abc123")
		.assert_header_contains("content-type", "json");
}

#[test]
fn test_response_assertions_json() {
	let body = serde_json::json!({
		"name": "John",
		"age": 30,
		"items": ["a", "b", "c"]
	});
	let body_bytes = Bytes::from(serde_json::to_vec(&body).unwrap());
	let response = MurTestResponse::new(StatusCode::OK, HashMap::new(), body_bytes);

	response
		.assert_json_valid()
		.assert_json_has("name")
		.assert_json_eq("age", 30)
		.assert_json_path("name", "John");
}

#[test]
fn test_case_insensitive_headers() {
	let mut headers = HashMap::new();
	headers.insert("Content-Type".to_string(), "application/json".to_string());

	let response = MurTestResponse::new(StatusCode::OK, headers, Bytes::new());
	assert!(response.header("content-type").is_some());
	assert!(response.header("Content-Type").is_some());
	assert!(response.header("CONTENT-TYPE").is_some());
}
