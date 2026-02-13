// TODO: analyse
use serde::{de::DeserializeOwned, Serialize};
use std::io::Write;

#[derive(Debug)]
pub enum JsonError {
	Simd(simd_json::Error),
	Serde(serde_json::Error),
	Utf8(std::str::Utf8Error),
}

impl std::fmt::Display for JsonError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			JsonError::Simd(e) => write!(f, "SIMD JSON error: {}", e),
			JsonError::Serde(e) => write!(f, "JSON error: {}", e),
			JsonError::Utf8(e) => write!(f, "UTF-8 error: {}", e),
		}
	}
}

impl std::error::Error for JsonError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			JsonError::Simd(e) => Some(e),
			JsonError::Serde(e) => Some(e),
			JsonError::Utf8(e) => Some(e),
		}
	}
}

impl From<simd_json::Error> for JsonError {
	#[inline]
	fn from(err: simd_json::Error) -> Self {
		JsonError::Simd(err)
	}
}

impl From<serde_json::Error> for JsonError {
	#[inline]
	fn from(err: serde_json::Error) -> Self {
		JsonError::Serde(err)
	}
}

impl From<std::str::Utf8Error> for JsonError {
	#[inline]
	fn from(err: std::str::Utf8Error) -> Self {
		JsonError::Utf8(err)
	}
}

pub type JsonResult<T> = Result<T, JsonError>;

#[inline]
pub fn from_slice<T: DeserializeOwned>(slice: &mut [u8]) -> JsonResult<T> {
	simd_json::from_slice(slice).map_err(JsonError::from)
}

#[inline]
pub fn from_slice_copy<T: DeserializeOwned>(slice: &[u8]) -> JsonResult<T> {
	let mut owned = slice.to_vec();
	simd_json::from_slice(&mut owned).map_err(JsonError::from)
}

#[inline]
pub fn from_str<T: DeserializeOwned>(s: &str) -> JsonResult<T> {
	let mut bytes = s.as_bytes().to_vec();
	simd_json::from_slice(&mut bytes).map_err(JsonError::from)
}

#[inline]
pub fn from_vec<T: DeserializeOwned>(mut vec: Vec<u8>) -> JsonResult<T> {
	simd_json::from_slice(&mut vec).map_err(JsonError::from)
}

#[inline]
pub fn from_bytes<T: DeserializeOwned>(bytes: hyper::body::Bytes) -> JsonResult<T> {
	let mut vec = bytes.to_vec();
	simd_json::from_slice(&mut vec).map_err(JsonError::from)
}

#[inline]
pub fn to_string<T: Serialize>(value: &T) -> JsonResult<String> {
	serde_json::to_string(value).map_err(JsonError::from)
}

#[inline]
pub fn to_string_pretty<T: Serialize>(value: &T) -> JsonResult<String> {
	serde_json::to_string_pretty(value).map_err(JsonError::from)
}

#[inline]
pub fn to_vec<T: Serialize>(value: &T) -> JsonResult<Vec<u8>> {
	serde_json::to_vec(value).map_err(JsonError::from)
}

#[inline]
pub fn to_vec_pretty<T: Serialize>(value: &T) -> JsonResult<Vec<u8>> {
	serde_json::to_vec_pretty(value).map_err(JsonError::from)
}

#[inline]
pub fn to_writer<W: Write, T: Serialize>(writer: W, value: &T) -> JsonResult<()> {
	serde_json::to_writer(writer, value).map_err(JsonError::from)
}

#[inline]
pub fn to_bytes<T: Serialize>(value: &T) -> JsonResult<hyper::body::Bytes> {
	let vec = serde_json::to_vec(value)?;
	Ok(hyper::body::Bytes::from(vec))
}

#[inline]
pub fn is_valid_json(slice: &[u8]) -> bool {
	let mut owned = slice.to_vec();
	let result = simd_json::to_borrowed_value(&mut owned).is_ok();
	result
}

#[inline]
pub fn parse_value(slice: &mut [u8]) -> JsonResult<simd_json::OwnedValue> {
	simd_json::to_owned_value(slice).map_err(JsonError::from)
}

#[inline]
pub fn value_to_owned(value: serde_json::Value) -> JsonResult<simd_json::OwnedValue> {
	let s = value.to_string();
	let mut bytes = s.into_bytes();
	simd_json::to_owned_value(&mut bytes).map_err(JsonError::from)
}

#[macro_export]
macro_rules! mur_json_value {
    ($($json:tt)+) => {
        serde_json::json!($($json)+)
    };
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde::Deserialize;

	#[derive(Debug, Serialize, Deserialize, PartialEq)]
	struct TestStruct {
		name: String,
		value: i32,
	}

	#[test]
	fn test_from_slice() {
		let mut data = br#"{"name":"test","value":42}"#.to_vec();
		let result: TestStruct = from_slice(&mut data).unwrap();
		assert_eq!(result.name, "test");
		assert_eq!(result.value, 42);
	}

	#[test]
	fn test_from_slice_copy() {
		let data = br#"{"name":"test","value":42}"#;
		let result: TestStruct = from_slice_copy(data).unwrap();
		assert_eq!(result.name, "test");
		assert_eq!(result.value, 42);
	}

	#[test]
	fn test_from_str() {
		let json = r#"{"name":"hello","value":100}"#;
		let result: TestStruct = from_str(json).unwrap();
		assert_eq!(result.name, "hello");
		assert_eq!(result.value, 100);
	}

	#[test]
	fn test_to_string() {
		let test = TestStruct {
			name: "world".into(),
			value: 200,
		};
		let json = to_string(&test).unwrap();
		assert!(json.contains("world"));
		assert!(json.contains("200"));
	}

	#[test]
	fn test_to_vec() {
		let test = TestStruct {
			name: "bytes".into(),
			value: 300,
		};
		let bytes = to_vec(&test).unwrap();
		assert!(!bytes.is_empty());
	}

	#[test]
	fn test_roundtrip() {
		let original = TestStruct {
			name: "roundtrip".into(),
			value: 999,
		};
		let json = to_string(&original).unwrap();
		let parsed: TestStruct = from_str(&json).unwrap();
		assert_eq!(original, parsed);
	}

	#[test]
	fn test_is_valid_json() {
		assert!(is_valid_json(b"{}"));
		assert!(is_valid_json(b"[]"));
		assert!(is_valid_json(br#"{"key":"value"}"#));
		assert!(!is_valid_json(b"not json"));
		assert!(!is_valid_json(b"{invalid}"));
	}

	#[test]
	fn test_invalid_json_error() {
		let result: JsonResult<TestStruct> = from_str("not valid json");
		assert!(result.is_err());
	}
}
