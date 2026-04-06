#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossDomainPolicy {
	None,
	MasterOnly,
	ByContentType,
	ByFtpFilename,
	All,
}

impl CrossDomainPolicy {
	pub fn as_str(&self) -> &'static str {
		match self {
			CrossDomainPolicy::None => "none",
			CrossDomainPolicy::MasterOnly => "master-only",
			CrossDomainPolicy::ByContentType => "by-content-type",
			CrossDomainPolicy::ByFtpFilename => "by-ftp-filename",
			CrossDomainPolicy::All => "all",
		}
	}
}

impl std::fmt::Display for CrossDomainPolicy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
