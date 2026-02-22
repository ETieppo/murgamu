use super::*;
use hyper::body::Bytes;

#[test]
fn test_mur_json_deref() {
	let json = MurJson(42);
	assert_eq!(*json, 42);
}

#[test]
fn test_mur_json_into_inner() {
	let json = MurJson(String::from("test"));
	assert_eq!(json.into_inner(), "test");
}

#[test]
fn test_mur_path_deref() {
	let path = MurPath(String::from("123"));
	assert_eq!(&*path, "123");
}

#[test]
fn test_mur_query_deref() {
	#[derive(Debug, PartialEq)]
	struct Query {
		page: i32,
	}
	let query = MurQuery(Query { page: 1 });
	assert_eq!(query.page, 1);
}

#[test]
fn test_mur_header() {
	let header = MurHeader::new("Bearer token123");
	assert_eq!(header.as_str(), "Bearer token123");
	assert!(!header.is_empty());
}

#[test]
fn test_mur_body_len() {
	let body = MurBody::new(Bytes::from("Hello"));
	assert_eq!(body.len(), 5);
	assert!(!body.is_empty());
}

#[test]
fn test_mur_text() {
	let text = MurText::new("Hello World");
	assert_eq!(text.as_str(), "Hello World");
	assert_eq!(text.len(), 11);
}

#[test]
fn test_mur_query_param() {
	let param: MurQueryParam<i32> = MurQueryParam::new(Some(42));
	assert!(param.is_present());
	assert_eq!(param.value, Some(42));
}

#[test]
fn test_mur_query_param_unwrap_or() {
	let param: MurQueryParam<i32> = MurQueryParam::new(None);
	assert!(!param.is_present());
	assert_eq!(param.unwrap_or(10), 10);
}
