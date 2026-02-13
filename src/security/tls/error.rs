use std::io;

#[derive(Debug)]
pub enum MurTlsError {
	CertificateRead(std::io::Error),
	CertificateParse(String),
	PrivateKeyRead(std::io::Error),
	PrivateKeyParse(String),
	NoPrivateKey,
	NoCertificates,
	ConfigBuild(String),
	InvalidVersion(String),
	General(String),
}

impl std::fmt::Display for MurTlsError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MurTlsError::CertificateRead(e) => write!(f, "Failed to read certificate: {}", e),
			MurTlsError::CertificateParse(e) => write!(f, "Failed to parse certificate: {}", e),
			MurTlsError::PrivateKeyRead(e) => write!(f, "Failed to read private key: {}", e),
			MurTlsError::PrivateKeyParse(e) => write!(f, "Failed to parse private key: {}", e),
			MurTlsError::NoPrivateKey => write!(f, "No private key found in file"),
			MurTlsError::NoCertificates => write!(f, "No certificates found in file"),
			MurTlsError::ConfigBuild(e) => write!(f, "Failed to build TLS config: {}", e),
			MurTlsError::InvalidVersion(e) => write!(f, "Invalid TLS version: {}", e),
			MurTlsError::General(e) => write!(f, "TLS error: {}", e),
		}
	}
}

impl std::error::Error for MurTlsError {}

impl From<MurTlsError> for io::Error {
	fn from(err: MurTlsError) -> Self {
		io::Error::other(err)
	}
}
