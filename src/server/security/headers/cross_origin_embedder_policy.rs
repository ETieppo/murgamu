#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossOriginEmbedderPolicy {
	UnsafeNone,
	RequireCorp,
	CredentialLess,
}

impl CrossOriginEmbedderPolicy {
	pub fn as_str(&self) -> &'static str {
		match self {
			CrossOriginEmbedderPolicy::UnsafeNone => "unsafe-none",
			CrossOriginEmbedderPolicy::RequireCorp => "require-corp",
			CrossOriginEmbedderPolicy::CredentialLess => "credentialless",
		}
	}
}

impl std::fmt::Display for CrossOriginEmbedderPolicy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
