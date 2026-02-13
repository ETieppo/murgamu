pub mod base;
pub mod env;
pub mod provider;
pub mod source;

pub use base::{MurConfig, MurConfigBuilder, MurConfigError, MurConfigResult};
pub use env::{MurEnv, MurEnvProfile};
pub use provider::{MurConfigProvider, MurConfigProviderBuilder, MurConfigService, MurFromConfig};

use crate::core::error::MurError;

pub fn mur_load_config() -> MurConfig {
	MurConfig::from_env()
}

pub fn mur_load_config_required(required_keys: &[&str]) -> MurConfigResult<MurConfig> {
	let config = MurConfig::from_env();
	config.validate_required(required_keys)?;
	Ok(config)
}

pub fn mur_env(key: &'static str) -> Result<String, MurError> {
	std::env::var(key).map_err(|_| MurError::NoEnv(key))
}

pub fn mur_env_or(key: &str, default: &str) -> String {
	std::env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn mur_env_parse<T: std::str::FromStr>(key: &str) -> Option<T> {
	std::env::var(key).ok().and_then(|v| v.parse().ok())
}

pub fn mur_env_parse_or<T: std::str::FromStr>(key: &str, default: T) -> T {
	mur_env_parse(key).unwrap_or(default)
}

pub fn mur_env_is_true(key: &str) -> bool {
	std::env::var(key)
		.map(|v| {
			let v = v.to_lowercase();
			v == "1" || v == "true" || v == "yes" || v == "on"
		})
		.unwrap_or(false)
}

pub fn mur_current_env() -> MurEnvProfile {
	MurEnv::current()
}

pub fn mur_is_development() -> bool {
	mur_current_env().is_development()
}

pub fn mur_is_production() -> bool {
	mur_current_env().is_production()
}

pub fn mur_is_test() -> bool {
	mur_current_env().is_test()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_mur_env_or() {
		let value = mur_env_or("NONEXISTENT_KEY_12345", "default");
		assert_eq!(value, "default");
	}

	#[test]
	fn test_mur_env_parse_or() {
		let value: u16 = mur_env_parse_or("NONEXISTENT_KEY_12345", 3000);
		assert_eq!(value, 3000);
	}

	#[test]
	fn test_mur_env_is_true() {
		assert!(!mur_env_is_true("NONEXISTENT_KEY_12345"));
	}
}
