use super::MurConfigProvider;
use crate::config::MurConfig;

#[derive(Debug, Clone, Default)]
pub struct MurConfigProviderBuilder {
	pub env_files: Vec<String>,
	pub required_keys: Vec<String>,
	pub defaults: Vec<(String, String)>,
	pub secrets: Vec<String>,
}

impl MurConfigProviderBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn env_file(mut self, path: &str) -> Self {
		self.env_files.push(path.to_string());
		self
	}

	pub fn required(mut self, key: &str) -> Self {
		self.required_keys.push(key.to_string());
		self
	}

	pub fn set_default(mut self, key: &str, value: &str) -> Self {
		self.defaults.push((key.to_string(), value.to_string()));
		self
	}

	pub fn secret(mut self, key: &str) -> Self {
		self.secrets.push(key.to_string());
		self
	}

	pub fn build(self) -> MurConfigProvider {
		let env_files = self.env_files;
		let required_keys = self.required_keys;
		let defaults = self.defaults;
		let secrets = self.secrets;

		MurConfigProvider::with_builder(move || {
			let mut config = MurConfig::from_env();

			for (key, value) in &defaults {
				config.set_default(key, value);
			}

			for path in &env_files {
				let _ = config.load_file(path);
			}

			for key in &secrets {
				config.mark_secret(key);
			}

			let required_refs: Vec<&str> = required_keys.iter().map(|s| s.as_str()).collect();
			if let Err(e) = config.validate_required(&required_refs) {
				eprintln!("Configuration warning: {}", e);
			}

			config
		})
	}
}
