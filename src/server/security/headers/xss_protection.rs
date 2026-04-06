#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XssProtection {
	Disabled,
	Enabled,
	EnabledBlock,
}

impl XssProtection {
	pub fn as_str(&self) -> &'static str {
		match self {
			XssProtection::Disabled => "0",
			XssProtection::Enabled => "1",
			XssProtection::EnabledBlock => "1; mode=block",
		}
	}
}

impl std::fmt::Display for XssProtection {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
