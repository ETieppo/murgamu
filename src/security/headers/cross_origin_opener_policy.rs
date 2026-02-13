#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossOriginOpenerPolicy {
	UnsafeNone,
	SameOrigin,
	SameOriginAllowPopups,
}

impl CrossOriginOpenerPolicy {
	pub fn as_str(&self) -> &'static str {
		match self {
			CrossOriginOpenerPolicy::UnsafeNone => "unsafe-none",
			CrossOriginOpenerPolicy::SameOrigin => "same-origin",
			CrossOriginOpenerPolicy::SameOriginAllowPopups => "same-origin-allow-popups",
		}
	}
}

impl std::fmt::Display for CrossOriginOpenerPolicy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
