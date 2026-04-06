use super::acceptor;
use super::config;
use super::config_builder;
use super::error;
use super::version;
use rcgen::{CertifiedKey, generate_simple_self_signed};

#[cfg(all(feature = "tls", test))]
pub fn generate_self_signed(
	domain: &str,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
	let subject_alt_names = vec![domain.to_string(), "localhost".to_string()];
	let CertifiedKey { cert, signing_key } = generate_simple_self_signed(subject_alt_names)?;

	Ok((cert.pem(), signing_key.serialize_pem()))
}

#[cfg(all(test, feature = "tls"))]
mod tests {
	use super::*;
	use acceptor::MurTlsAcceptor;
	use config::MurTlsConfig;
	use config_builder::MurTlsConfigBuilder;
	use error::MurTlsError;
	use version::MurTlsVersion;

	#[test]
	fn test_tls_version_default() {
		assert_eq!(MurTlsVersion::default(), MurTlsVersion::Tls13);
	}

	#[test]
	fn test_tls_error_display() {
		let err = MurTlsError::NoPrivateKey;
		assert_eq!(err.to_string(), "No private key found in file");

		let err = MurTlsError::NoCertificates;
		assert_eq!(err.to_string(), "No certificates found in file");
	}

	#[test]
	fn test_tls_config_builder() {
		let builder = MurTlsConfigBuilder::new()
			.min_version(MurTlsVersion::Tls12)
			.max_version(MurTlsVersion::Tls13)
			.http2_only()
			.session_resumption(true);

		assert_eq!(builder.min_version, MurTlsVersion::Tls12);
		assert_eq!(builder.max_version, MurTlsVersion::Tls13);
		assert_eq!(builder.alpn_protocols, vec![b"h2".to_vec()]);
	}

	#[test]
	fn test_generate_self_signed() {
		let result = generate_self_signed("localhost");
		assert!(result.is_ok());

		let (cert_pem, key_pem) = result.unwrap();
		assert!(cert_pem.contains("BEGIN CERTIFICATE"));
		assert!(key_pem.contains("BEGIN PRIVATE KEY"));
	}

	#[test]
	fn test_load_from_generated_pem() {
		let (cert_pem, key_pem) = generate_self_signed("localhost").unwrap();
		let config = MurTlsConfig::from_pem(&cert_pem, &key_pem);
		assert!(config.is_ok());

		let config = config.unwrap();
		assert!(!config.certs.is_empty());
		assert!(!config.key_der.is_empty());
	}

	#[test]
	fn test_build_acceptor() {
		let (cert_pem, key_pem) = generate_self_signed("localhost").unwrap();
		let config = MurTlsConfig::from_pem(&cert_pem, &key_pem).unwrap();
		let acceptor = MurTlsAcceptor::new(&config);

		assert!(acceptor.is_ok());
	}
}
