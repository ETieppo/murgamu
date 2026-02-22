use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum AllowedOrigins {
	Any,
	List(HashSet<String>),
	Mirror,
}

impl AllowedOrigins {
	pub fn is_allowed(&self, origin: &str) -> bool {
		match self {
			AllowedOrigins::Any => true,
			AllowedOrigins::Mirror => true,
			AllowedOrigins::List(origins) => origins.contains(origin),
		}
	}

	pub fn header_value(&self, request_origin: Option<&str>) -> Option<String> {
		match self {
			AllowedOrigins::Any => Some("*".to_string()),
			AllowedOrigins::Mirror => request_origin.map(|o| o.to_string()),
			AllowedOrigins::List(origins) => {
				if let Some(origin) = request_origin {
					if origins.contains(origin) {
						Some(origin.to_string())
					} else {
						None
					}
				} else {
					None
				}
			}
		}
	}
}
