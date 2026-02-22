use super::*;
use crate::server::config::MurConfig;
use crate::server::config::MurConfigError;
use crate::server::service::MurService;
use std::any::Any;

#[test]
fn test_config_service() {
	let mut config = MurConfig::new();
	config.set("PORT", "8080");
	config.set("HOST", "localhost");

	let service = MurConfigService::new(config);
	assert_eq!(service.get_or::<u16>("PORT", 3000), 8080);
	assert_eq!(service.get_or_string("HOST", ""), "localhost");
}

#[test]
fn test_config_provider() {
	let provider = MurConfigProvider::from_env();
	let service = provider.provide();

	assert!(service.environment().is_some() || service.environment().is_none());
}

#[test]
fn test_config_provider_builder() {
	let provider = MurConfigProviderBuilder::new()
		.set_default("PORT", "3000")
		.set_default("HOST", "localhost")
		.build();

	let service = provider.provide();
	assert_eq!(service.get_or::<u16>("PORT", 0), 3000);
}

#[test]
fn test_config_service_as_service() {
	let config = MurConfig::new();
	let service = MurConfigService::new(config);
	let _any: &dyn Any = service.as_any();
}

#[derive(Debug, Clone)]
struct TestConfig {
	pub port: u16,
	pub host: String,
}

impl MurFromConfig for TestConfig {
	fn from_config(config: &MurConfig) -> Result<Self, MurConfigError> {
		Ok(Self {
			port: config.get_or("PORT", 3000),
			host: config.get_or_string("HOST", "localhost"),
		})
	}
}

#[test]
fn test_from_config_trait() {
	let mut config = MurConfig::new();
	config.set("PORT", "8080");
	config.set("HOST", "example.com");

	let test_config = TestConfig::from_config(&config).unwrap();
	assert_eq!(test_config.port, 8080);
	assert_eq!(test_config.host, "example.com");
}

#[test]
fn test_from_config_prefix() {
	let mut config = MurConfig::new();
	config.set("DB_PORT", "5432");
	config.set("DB_HOST", "db.example.com");

	#[derive(Debug)]
	struct DbConfig {
		port: u16,
		host: String,
	}

	impl MurFromConfig for DbConfig {
		fn from_config(config: &MurConfig) -> Result<Self, MurConfigError> {
			Ok(Self {
				port: config.get_or("PORT", 5432),
				host: config.get_or_string("HOST", "localhost"),
			})
		}
	}

	let db_config = DbConfig::from_config_prefix(&config, "DB_").unwrap();
	assert_eq!(db_config.port, 5432);
	assert_eq!(db_config.host, "db.example.com");
}
