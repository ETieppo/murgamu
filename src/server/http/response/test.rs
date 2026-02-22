use http::StatusCode;

use super::*;

#[test]
fn test_response_builder_json() {
	let response = MurResponseBuilder::new()
		.status(StatusCode::OK)
		.json(serde_json::json!({"key": "value"}));

	assert!(response.is_ok());
	let response = response.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
	assert_eq!(
		response.headers().get("Content-Type").unwrap(),
		"application/json"
	);
}

#[test]
fn test_response_builder_text() {
	let response = MurResponseBuilder::new()
		.status(StatusCode::OK)
		.text("Hello World");

	assert!(response.is_ok());
	let response = response.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
	assert!(response
		.headers()
		.get("Content-Type")
		.unwrap()
		.to_str()
		.unwrap()
		.contains("text/plain"));
}

#[test]
fn test_http_response_ok() {
	let response = MurHttpResponse::ok().json(serde_json::json!({"status": "ok"}));
	assert!(response.is_ok());
	assert_eq!(response.unwrap().status(), StatusCode::OK);
}

#[test]
fn test_http_response_created() {
	let response = MurHttpResponse::created().json(serde_json::json!({"id": 123}));
	assert!(response.is_ok());
	assert_eq!(response.unwrap().status(), StatusCode::CREATED);
}

#[test]
fn test_http_response_not_found() {
	let response = MurHttpResponse::not_found().json(serde_json::json!({"error": "Not found"}));
	assert!(response.is_ok());
	assert_eq!(response.unwrap().status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_http_response_no_content() {
	let response = MurHttpResponse::no_content();
	assert!(response.is_ok());
	assert_eq!(response.unwrap().status(), StatusCode::NO_CONTENT);
}

#[test]
fn test_response_builder_with_headers() {
	let response = MurResponseBuilder::new()
		.status(StatusCode::OK)
		.header("X-Custom-Header", "custom-value")
		.header("X-Another", "another-value")
		.json(serde_json::json!({}));

	assert!(response.is_ok());
	let response = response.unwrap();
	assert_eq!(
		response.headers().get("X-Custom-Header").unwrap(),
		"custom-value"
	);
	assert_eq!(
		response.headers().get("X-Another").unwrap(),
		"another-value"
	);
}

#[test]
fn test_into_response_string() {
	let response = "Hello World".into_response();
	assert!(response.is_ok());
	assert_eq!(response.unwrap().status(), StatusCode::OK);
}

#[test]
fn test_into_response_unit() {
	let response = ().into_response();
	assert!(response.is_ok());
	assert_eq!(response.unwrap().status(), StatusCode::NO_CONTENT);
}

#[test]
fn test_into_response_status_code() {
	let response = StatusCode::ACCEPTED.into_response();
	assert!(response.is_ok());
	assert_eq!(response.unwrap().status(), StatusCode::ACCEPTED);
}
