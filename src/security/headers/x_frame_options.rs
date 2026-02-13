#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XFrameOptions {
	Deny,
	SameOrigin,
}

impl XFrameOptions {
	pub fn as_str(&self) -> &'static str {
		match self {
			XFrameOptions::Deny => "DENY",
			XFrameOptions::SameOrigin => "SAMEORIGIN",
		}
	}
}

impl std::fmt::Display for XFrameOptions {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
