use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum AllowedHeaders {
	Any,
	List(HashSet<String>),
	Mirror,
}

impl AllowedHeaders {
	pub fn is_allowed(&self, header: &str) -> bool {
		match self {
			AllowedHeaders::Any => true,
			AllowedHeaders::Mirror => true,
			AllowedHeaders::List(headers) => headers.contains(&header.to_lowercase()),
		}
	}

	pub fn header_value(&self, requested_headers: Option<&str>) -> Option<String> {
		match self {
			AllowedHeaders::Any => Some("*".to_string()),
			AllowedHeaders::Mirror => requested_headers.map(|h| h.to_string()),
			AllowedHeaders::List(headers) => {
				if headers.is_empty() {
					None
				} else {
					Some(headers.iter().cloned().collect::<Vec<_>>().join(", "))
				}
			}
		}
	}
}
