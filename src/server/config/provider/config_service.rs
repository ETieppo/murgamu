use crate::server::config::MurConfig;
use crate::server::config::MurConfigError;
use crate::server::service::MurService;
use std::any::Any;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MurConfigService {
	config: Arc<MurConfig>,
}

impl MurConfigService {
	pub fn new(config: MurConfig) -> Self {
		Self {
			config: Arc::new(config),
		}
	}

	pub fn from_arc(config: Arc<MurConfig>) -> Self {
		Self { config }
	}

	pub fn from_env() -> Self {
		Self::new(MurConfig::from_env())
	}

	pub fn config(&self) -> &MurConfig {
		&self.config
	}

	pub fn into_arc(self) -> Arc<MurConfig> {
		self.config
	}

	pub fn get(&self, key: &str) -> Option<&String> {
		self.config.get(key)
	}

	pub fn get_or_string(&self, key: &str, default: &str) -> String {
		self.config.get_or_string(key, default)
	}

	pub fn get_or<T: std::str::FromStr>(&self, key: &str, default: T) -> T {
		self.config.get_or(key, default)
	}

	pub fn get_required(&self, key: &str) -> Result<String, MurConfigError> {
		self.config.get_required(key)
	}

	pub fn get_required_as<T: std::str::FromStr>(&self, key: &str) -> Result<T, MurConfigError> {
		self.config.get_required_as(key)
	}

	pub fn get_bool(&self, key: &str) -> Option<bool> {
		self.config.get_bool(key)
	}

	pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
		self.config.get_bool_or(key, default)
	}

	pub fn get_list(&self, key: &str) -> Option<Vec<String>> {
		self.config.get_list(key)
	}

	pub fn get_list_or(&self, key: &str, default: Vec<String>) -> Vec<String> {
		self.config.get_list_or(key, default)
	}

	pub fn get_duration(&self, key: &str) -> Option<std::time::Duration> {
		self.config.get_duration(key)
	}

	pub fn get_duration_or(&self, key: &str, default: std::time::Duration) -> std::time::Duration {
		self.config.get_duration_or(key, default)
	}

	pub fn get_size_bytes(&self, key: &str) -> Option<u64> {
		self.config.get_size_bytes(key)
	}

	pub fn get_size_bytes_or(&self, key: &str, default: u64) -> u64 {
		self.config.get_size_bytes_or(key, default)
	}

	pub fn has(&self, key: &str) -> bool {
		self.config.has(key)
	}

	pub fn is_secret(&self, key: &str) -> bool {
		self.config.is_secret(key)
	}

	pub fn environment(&self) -> Option<&String> {
		self.config.environment()
	}

	pub fn subset(&self, prefix: &str) -> MurConfig {
		self.config.subset(prefix)
	}
}

impl MurService for MurConfigService {
	fn as_any(&self) -> &dyn Any {
		self
	}

	fn on_init(&self) {
		if let Some(env) = self.environment() {
			eprintln!("Configuration loaded for environment: {}", env);
		}
	}
}
