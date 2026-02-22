use super::MurConfig;
use super::MurConfigResult;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Default)]
pub struct MurConfigBuilder {
	env_files: Vec<String>,
	required_keys: Vec<String>,
	defaults: HashMap<String, String>,
	secrets: Vec<String>,
	load_env: bool,
	values: HashMap<String, String>,
}

impl MurConfigBuilder {
	pub fn new() -> Self {
		Self {
			load_env: true,
			..Default::default()
		}
	}

	pub fn env_file(mut self, path: &str) -> Self {
		self.env_files.push(path.to_string());
		self
	}

	pub fn env_file_for_env(mut self, path: &str) -> Self {
		if Path::new(path).exists() {
			self.env_files.push(path.to_string());
		}
		self
	}

	pub fn required(mut self, key: &str) -> Self {
		self.required_keys.push(key.to_string());
		self
	}

	pub fn default(mut self, key: &str, value: &str) -> Self {
		self.defaults.insert(key.to_string(), value.to_string());
		self
	}

	pub fn secret(mut self, key: &str) -> Self {
		self.secrets.push(key.to_string());
		self
	}

	pub fn set(mut self, key: &str, value: &str) -> Self {
		self.values.insert(key.to_string(), value.to_string());
		self
	}

	pub fn no_env(mut self) -> Self {
		self.load_env = false;
		self
	}

	pub fn build(self) -> MurConfigResult<MurConfig> {
		let mut config = MurConfig::new();

		for (key, value) in self.defaults {
			config.set(key, value);
		}

		for path in &self.env_files {
			config.load_file(path)?;
		}

		if self.load_env {
			config.load_env();
		}

		for (key, value) in self.values {
			config.set(key, value);
		}

		for key in self.secrets {
			config.mark_secret(&key);
		}

		let required_refs: Vec<&str> = self.required_keys.iter().map(|s| s.as_str()).collect();
		config.validate_required(&required_refs)?;

		Ok(config)
	}
}
