use super::config_builder::MurTlsConfigBuilder;
use super::error::MurTlsError;
use super::version::MurTlsVersion;
use super::MurTlsLoader;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::ServerConfig;
use std::path::Path;
use std::sync::Arc;
use tokio_rustls::TlsAcceptor;

pub struct MurTlsConfig {
	pub(crate) certs: Vec<CertificateDer<'static>>,
	pub(crate) key_der: Vec<u8>,
	pub min_version: MurTlsVersion,
	pub max_version: MurTlsVersion,
	pub alpn_protocols: Vec<Vec<u8>>,
	pub client_auth: bool,
	pub(crate) client_ca_certs: Option<Vec<CertificateDer<'static>>>,
	pub session_resumption: bool,
	pub ocsp_response: Option<Vec<u8>>,
}

impl Clone for MurTlsConfig {
	fn clone(&self) -> Self {
		Self {
			certs: self.certs.clone(),
			key_der: self.key_der.clone(),
			min_version: self.min_version,
			max_version: self.max_version,
			alpn_protocols: self.alpn_protocols.clone(),
			client_auth: self.client_auth,
			client_ca_certs: self.client_ca_certs.clone(),
			session_resumption: self.session_resumption,
			ocsp_response: self.ocsp_response.clone(),
		}
	}
}

impl std::fmt::Debug for MurTlsConfig {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurTlsConfig")
			.field("certs_count", &self.certs.len())
			.field("min_version", &self.min_version)
			.field("max_version", &self.max_version)
			.field("alpn_protocols", &self.alpn_protocols.len())
			.field("client_auth", &self.client_auth)
			.field("session_resumption", &self.session_resumption)
			.finish()
	}
}

impl MurTlsConfig {
	pub fn from_pem_files(
		cert_path: impl AsRef<Path>,
		key_path: impl AsRef<Path>,
	) -> Result<Self, MurTlsError> {
		let certs = MurTlsLoader::certs(cert_path)?;
		let key = MurTlsLoader::private_key(key_path)?;
		let key_der = key.secret_der().to_vec();

		Ok(Self {
			certs,
			key_der,
			min_version: MurTlsVersion::Tls12,
			max_version: MurTlsVersion::Tls13,
			alpn_protocols: vec![b"h2".to_vec(), b"http/1.1".to_vec()],
			client_auth: false,
			client_ca_certs: None,
			session_resumption: true,
			ocsp_response: None,
		})
	}

	pub fn from_pem(cert_pem: &str, key_pem: &str) -> Result<Self, MurTlsError> {
		let certs = MurTlsLoader::certs_from_pem(cert_pem)?;
		let key = MurTlsLoader::private_key_from_pem(key_pem)?;
		let key_der = key.secret_der().to_vec();

		Ok(Self {
			certs,
			key_der,
			min_version: MurTlsVersion::Tls12,
			max_version: MurTlsVersion::Tls13,
			alpn_protocols: vec![b"h2".to_vec(), b"http/1.1".to_vec()],
			client_auth: false,
			client_ca_certs: None,
			session_resumption: true,
			ocsp_response: None,
		})
	}

	pub fn from_der(cert_der: Vec<Vec<u8>>, key_der_bytes: Vec<u8>) -> Result<Self, MurTlsError> {
		let certs: Vec<CertificateDer<'static>> =
			cert_der.into_iter().map(CertificateDer::from).collect();

		Ok(Self {
			certs,
			key_der: key_der_bytes,
			min_version: MurTlsVersion::Tls12,
			max_version: MurTlsVersion::Tls13,
			alpn_protocols: vec![b"h2".to_vec(), b"http/1.1".to_vec()],
			client_auth: false,
			client_ca_certs: None,
			session_resumption: true,
			ocsp_response: None,
		})
	}

	pub fn builder() -> MurTlsConfigBuilder {
		MurTlsConfigBuilder::new()
	}

	pub fn min_version(mut self, version: MurTlsVersion) -> Self {
		self.min_version = version;
		self
	}

	pub fn max_version(mut self, version: MurTlsVersion) -> Self {
		self.max_version = version;
		self
	}

	pub fn alpn_protocols<I, S>(mut self, protocols: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: AsRef<[u8]>,
	{
		self.alpn_protocols = protocols.into_iter().map(|p| p.as_ref().to_vec()).collect();
		self
	}

	pub fn client_auth(mut self, enabled: bool) -> Self {
		self.client_auth = enabled;
		self
	}

	pub fn client_ca_certs_path(mut self, path: impl AsRef<Path>) -> Result<Self, MurTlsError> {
		self.client_ca_certs = Some(MurTlsLoader::certs(path)?);
		self.client_auth = true;
		Ok(self)
	}

	pub fn session_resumption(mut self, enabled: bool) -> Self {
		self.session_resumption = enabled;
		self
	}

	pub fn ocsp_response(mut self, response: Vec<u8>) -> Self {
		self.ocsp_response = Some(response);
		self
	}

	pub fn build_server_config(&self) -> Result<ServerConfig, MurTlsError> {
		let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(self.key_der.clone()));
		let mut config = ServerConfig::builder()
			.with_no_client_auth()
			.with_single_cert(self.certs.clone(), key)
			.map_err(|e| MurTlsError::ConfigBuild(e.to_string()))?;

		config.alpn_protocols = self.alpn_protocols.clone();
		Ok(config)
	}

	pub fn build_acceptor(&self) -> Result<TlsAcceptor, MurTlsError> {
		let config = self.build_server_config()?;
		Ok(TlsAcceptor::from(Arc::new(config)))
	}
}

#[cfg(not(feature = "tls"))]
impl MurTlsConfig {
	pub fn from_pem_files(
		_cert_path: impl AsRef<std::path::Path>,
		_key_path: impl AsRef<std::path::Path>,
	) -> Result<Self, MurTlsError> {
		Err(MurTlsError::General(
			"TLS feature is not enabled. Add `tls` feature to Cargo.toml".to_string(),
		))
	}

	pub fn from_pem(_cert_pem: &str, _key_pem: &str) -> Result<Self, MurTlsError> {
		Err(MurTlsError::General(
			"TLS feature is not enabled. Add `tls` feature to Cargo.toml".to_string(),
		))
	}
}
