use crate::server::error::MurError;
use hyper::body::Bytes;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

pub struct MurParse;

impl MurParse {
	pub async fn body<T: DeserializeOwned>(body: Option<Bytes>) -> Result<T, String> {
		match body {
			Some(bytes) => serde_json::from_slice(&bytes).map_err(|e| e.to_string()),
			None => Err("No body provided".to_string()),
		}
	}

	pub fn json<T: DeserializeOwned>(body: Option<Bytes>) -> Result<T, MurError> {
		match body {
			Some(bytes) => serde_json::from_slice(&bytes).map_err(MurError::from),
			None => Err(MurError::BadRequest("No body provided".to_string())),
		}
	}

	pub fn text(body: Option<Bytes>) -> Result<String, MurError> {
		match body {
			Some(bytes) => String::from_utf8(bytes.to_vec())
				.map_err(|e| MurError::BadRequest(format!("Invalid UTF-8: {}", e))),
			None => Err(MurError::BadRequest("No body provided".to_string())),
		}
	}

	pub fn form(body: Option<Bytes>) -> Result<HashMap<String, String>, MurError> {
		match body {
			Some(bytes) => {
				let text = String::from_utf8(bytes.to_vec())
					.map_err(|e| MurError::BadRequest(format!("Invalid UTF-8: {}", e)))?;

				let mut params = HashMap::new();
				for pair in text.split('&') {
					if let Some((key, value)) = pair.split_once('=') {
						let key = urlencoding::decode(key)
							.map_err(|e| {
								MurError::BadRequest(format!("Invalid URL encoding: {}", e))
							})?
							.into_owned();
						let value = urlencoding::decode(value)
							.map_err(|e| {
								MurError::BadRequest(format!("Invalid URL encoding: {}", e))
							})?
							.into_owned();
						params.insert(key, value);
					}
				}
				Ok(params)
			}
			None => Err(MurError::BadRequest("No body provided".to_string())),
		}
	}

	pub fn form_typed<T: DeserializeOwned>(body: Option<Bytes>) -> Result<T, MurError> {
		match body {
			Some(bytes) => {
				let text = String::from_utf8(bytes.to_vec())
					.map_err(|e| MurError::BadRequest(format!("Invalid UTF-8: {}", e)))?;
				serde_urlencoded::from_str(&text)
					.map_err(|e| MurError::BadRequest(format!("Failed to parse form: {}", e)))
			}
			None => Err(MurError::BadRequest("No body provided".to_string())),
		}
	}
}
