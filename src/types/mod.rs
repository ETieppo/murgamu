mod aliases;
pub use crate::mur_http::{MurHttpResponse, MurIntoResponse, MurResExt, MurResponseBuilder};
pub use aliases::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MurMethod {
	Get,
	Post,
	Put,
	Delete,
	Patch,
	Head,
	Options,
}

impl MurMethod {
	pub fn as_str(&self) -> &str {
		match self {
			MurMethod::Get => "GET",
			MurMethod::Post => "POST",
			MurMethod::Put => "PUT",
			MurMethod::Delete => "DELETE",
			MurMethod::Patch => "PATCH",
			MurMethod::Head => "HEAD",
			MurMethod::Options => "OPTIONS",
		}
	}
}

impl From<&str> for MurMethod {
	fn from(s: &str) -> Self {
		match s.to_uppercase().as_str() {
			"GET" => MurMethod::Get,
			"POST" => MurMethod::Post,
			"PUT" => MurMethod::Put,
			"DELETE" => MurMethod::Delete,
			"PATCH" => MurMethod::Patch,
			"HEAD" => MurMethod::Head,
			"OPTIONS" => MurMethod::Options,
			_ => MurMethod::Get,
		}
	}
}

impl std::fmt::Display for MurMethod {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
