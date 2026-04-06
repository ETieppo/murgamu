use super::config::MurTlsConfig;
use super::error::MurTlsError;
use super::loader::MurTlsLoader;
use super::version::MurTlsVersion;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

#[derive(Default)]
pub struct MurTlsConfigBuilder {
	pub cert_path: Option<String>,
	pub key_path: Option<String>,
	pub cert_pem: Option<String>,
	pub key_pem: Option<String>,
	pub min_version: MurTlsVersion,
	pub max_version: MurTlsVersion,
	pub alpn_protocols: Vec<Vec<u8>>,
	pub client_auth: bool,
	pub client_ca_path: Option<String>,
	pub session_resumption: bool,
	pub ocsp_response: Option<Vec<u8>>,
}

impl MurTlsConfigBuilder {
	pub fn new() -> Self {
		Self {
			cert_path: None,
			key_path: None,
			cert_pem: None,
			key_pem: None,
			min_version: MurTlsVersion::Tls12,
			max_version: MurTlsVersion::Tls13,
			alpn_protocols: vec![b"h2".to_vec(), b"http/1.1".to_vec()],
			client_auth: false,
			client_ca_path: None,
			session_resumption: true,
			ocsp_response: None,
		}
	}

	pub fn cert_path(mut self, path: impl Into<String>) -> Self {
		self.cert_path = Some(path.into());
		self
	}

	pub fn key_path(mut self, path: impl Into<String>) -> Self {
		self.key_path = Some(path.into());
		self
	}

	pub fn cert_pem(mut self, pem: impl Into<String>) -> Self {
		self.cert_pem = Some(pem.into());
		self
	}

	pub fn key_pem(mut self, pem: impl Into<String>) -> Self {
		self.key_pem = Some(pem.into());
		self
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

	pub fn http2_only(mut self) -> Self {
		self.alpn_protocols = vec![b"h2".to_vec()];
		self
	}

	pub fn http1_only(mut self) -> Self {
		self.alpn_protocols = vec![b"http/1.1".to_vec()];
		self
	}

	pub fn client_auth(mut self, enabled: bool) -> Self {
		self.client_auth = enabled;
		self
	}

	pub fn client_ca_path(mut self, path: impl Into<String>) -> Self {
		self.client_ca_path = Some(path.into());
		self.client_auth = true;
		self
	}

	pub fn session_resumption(mut self, enabled: bool) -> Self {
		self.session_resumption = enabled;
		self
	}

	pub fn ocsp_response(mut self, response: Vec<u8>) -> Self {
		self.ocsp_response = Some(response);
		self
	}

	pub fn build(self) -> Result<MurTlsConfig, MurTlsError> {
		let (certs, key): (Vec<CertificateDer<'static>>, PrivateKeyDer<'static>) =
			if let (Some(cert_path), Some(key_path)) = (&self.cert_path, &self.key_path) {
				(
					MurTlsLoader::certs(cert_path)?,
					MurTlsLoader::private_key(key_path)?,
				)
			} else if let (Some(cert_pem), Some(key_pem)) = (&self.cert_pem, &self.key_pem) {
				(
					MurTlsLoader::certs_from_pem(cert_pem)?,
					MurTlsLoader::private_key_from_pem(key_pem)?,
				)
			} else {
				return Err(MurTlsError::General(
					"Either cert_path/key_path or cert_pem/key_pem must be set".to_string(),
				));
			};

		let key_der: Vec<u8> = key.secret_der().to_vec();
		let client_ca_certs = if let Some(ca_path) = &self.client_ca_path {
			Some(MurTlsLoader::certs(ca_path)?)
		} else {
			None
		};

		Ok(MurTlsConfig {
			certs,
			key_der,
			min_version: self.min_version,
			max_version: self.max_version,
			alpn_protocols: self.alpn_protocols,
			client_auth: self.client_auth,
			client_ca_certs,
			session_resumption: self.session_resumption,
			ocsp_response: self.ocsp_response,
		})
	}
}
