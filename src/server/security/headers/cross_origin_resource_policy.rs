#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossOriginResourcePolicy {
	CrossOrigin,
	SameSite,
	SameOrigin,
}

impl CrossOriginResourcePolicy {
	pub fn as_str(&self) -> &'static str {
		match self {
			CrossOriginResourcePolicy::CrossOrigin => "cross-origin",
			CrossOriginResourcePolicy::SameSite => "same-site",
			CrossOriginResourcePolicy::SameOrigin => "same-origin",
		}
	}
}

impl std::fmt::Display for CrossOriginResourcePolicy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
