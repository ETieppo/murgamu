use super::error::MurTlsError;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use std::path::Path;
use std::{fs::File, io::BufReader};

pub struct MurTlsLoader;

impl MurTlsLoader {
	pub fn certs(path: impl AsRef<Path>) -> Result<Vec<CertificateDer<'static>>, MurTlsError> {
		let file = File::open(path).map_err(MurTlsError::CertificateRead)?;
		let mut reader = BufReader::new(file);
		let certs: Vec<CertificateDer<'static>> = certs(&mut reader)
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| MurTlsError::CertificateParse(e.to_string()))?;

		if certs.is_empty() {
			return Err(MurTlsError::NoCertificates);
		}

		Ok(certs)
	}

	pub fn certs_from_pem(pem: &str) -> Result<Vec<CertificateDer<'static>>, MurTlsError> {
		let mut reader = BufReader::new(pem.as_bytes());
		let certs: Vec<CertificateDer<'static>> = certs(&mut reader)
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| MurTlsError::CertificateParse(e.to_string()))?;

		if certs.is_empty() {
			return Err(MurTlsError::NoCertificates);
		}

		Ok(certs)
	}

	pub fn private_key(path: impl AsRef<Path>) -> Result<PrivateKeyDer<'static>, MurTlsError> {
		let file = File::open(path).map_err(MurTlsError::PrivateKeyRead)?;
		let mut reader = BufReader::new(file);

		private_key(&mut reader)
			.map_err(|e| MurTlsError::PrivateKeyParse(e.to_string()))?
			.ok_or(MurTlsError::NoPrivateKey)
	}

	pub fn private_key_from_pem(pem: &str) -> Result<PrivateKeyDer<'static>, MurTlsError> {
		let mut reader = BufReader::new(pem.as_bytes());

		private_key(&mut reader)
			.map_err(|e| MurTlsError::PrivateKeyParse(e.to_string()))?
			.ok_or(MurTlsError::NoPrivateKey)
	}
}
