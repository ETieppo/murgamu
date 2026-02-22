use super::parse_duration;
use super::parse_size;
use super::MurConfigBuilder;
use super::MurConfigError;
use super::MurConfigResult;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MurConfig {
	pub values: HashMap<String, String>,
	metadata: ConfigMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigMetadata {
	pub loaded_files: Vec<String>,
	pub secret_keys: Vec<String>,
	pub environment: Option<String>,
}

impl MurConfig {
	pub fn new() -> Self {
		Self {
			values: HashMap::new(),
			metadata: ConfigMetadata::default(),
		}
	}

	pub fn from_map(values: HashMap<String, String>) -> Self {
		Self {
			values,
			metadata: ConfigMetadata::default(),
		}
	}

	pub fn from_env() -> Self {
		let mut config = Self::new();
		config.load_env();
		config.try_load_dotenv();
		config
	}

	pub fn builder() -> MurConfigBuilder {
		MurConfigBuilder::new()
	}

	pub fn load_env(&mut self) {
		for (key, value) in std::env::vars() {
			self.values.insert(key, value);
		}
	}

	fn try_load_dotenv(&mut self) {
		if Path::new(".env").exists() {
			let _ = self.load_file(".env");
		}

		let env = self.detect_environment();
		self.metadata.environment = Some(env.clone());

		let env_file = format!(".env.{}", env);
		if Path::new(&env_file).exists() {
			let _ = self.load_file(&env_file);
		}

		if Path::new(".env.local").exists() {
			let _ = self.load_file(".env.local");
		}

		for (key, value) in std::env::vars() {
			self.values.insert(key, value);
		}
	}

	fn detect_environment(&self) -> String {
		self.values
			.get("MUR_ENV")
			.or_else(|| self.values.get("APP_ENV"))
			.or_else(|| self.values.get("RUST_ENV"))
			.or_else(|| self.values.get("NODE_ENV"))
			.cloned()
			.unwrap_or_else(|| "development".to_string())
			.to_lowercase()
	}

	pub fn load_file(&mut self, path: &str) -> MurConfigResult<()> {
		let content = std::fs::read_to_string(path).map_err(|e| MurConfigError::FileError {
			path: path.to_string(),
			message: e.to_string(),
		})?;

		self.parse_dotenv(&content);
		self.metadata.loaded_files.push(path.to_string());
		Ok(())
	}

	pub fn parse_dotenv(&mut self, content: &str) {
		for line in content.lines() {
			let line = line.trim();

			if line.is_empty() || line.starts_with('#') {
				continue;
			}

			if let Some((key, value)) = line.split_once('=') {
				let key = key.trim().to_string();
				let value = Self::parse_value(value.trim());
				self.values.insert(key, value);
			}
		}
	}

	fn parse_value(value: &str) -> String {
		let value = value.trim();

		if (value.starts_with('"') && value.ends_with('"'))
			|| (value.starts_with('\'') && value.ends_with('\''))
		{
			let inner = &value[1..value.len() - 1];

			if value.starts_with('"') {
				return inner
					.replace("\\n", "\n")
					.replace("\\t", "\t")
					.replace("\\r", "\r")
					.replace("\\\"", "\"")
					.replace("\\\\", "\\");
			}
			return inner.to_string();
		}

		if let Some(idx) = value.find(" #") {
			return value[..idx].trim().to_string();
		}

		value.to_string()
	}

	pub fn get(&self, key: &str) -> Option<&String> {
		self.values.get(key)
	}

	pub fn get_or_string(&self, key: &str, default: &str) -> String {
		self.values
			.get(key)
			.cloned()
			.unwrap_or_else(|| default.to_string())
	}

	pub fn get_or<T: std::str::FromStr>(&self, key: &str, default: T) -> T {
		self.values
			.get(key)
			.and_then(|v| v.parse().ok())
			.unwrap_or(default)
	}

	pub fn get_required(&self, key: &str) -> MurConfigResult<String> {
		self.values
			.get(key)
			.cloned()
			.ok_or_else(|| MurConfigError::MissingKey(key.to_string()))
	}

	pub fn get_required_as<T: std::str::FromStr>(&self, key: &str) -> MurConfigResult<T> {
		let value = self.get_required(key)?;
		value.parse().map_err(|_| MurConfigError::ParseError {
			key: key.to_string(),
			message: format!(
				"Failed to parse '{}' as {}",
				value,
				std::any::type_name::<T>()
			),
		})
	}

	pub fn get_bool(&self, key: &str) -> Option<bool> {
		self.values.get(key).map(|v| {
			let v = v.to_lowercase();
			v == "1" || v == "true" || v == "yes" || v == "on"
		})
	}

	pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
		self.get_bool(key).unwrap_or(default)
	}

	pub fn get_list(&self, key: &str) -> Option<Vec<String>> {
		self.values.get(key).map(|v| {
			v.split(',')
				.map(|s| s.trim().to_string())
				.filter(|s| !s.is_empty())
				.collect()
		})
	}

	pub fn get_list_or(&self, key: &str, default: Vec<String>) -> Vec<String> {
		self.get_list(key).unwrap_or(default)
	}

	pub fn get_duration(&self, key: &str) -> Option<std::time::Duration> {
		self.values.get(key).and_then(|v| parse_duration(v))
	}

	pub fn get_duration_or(&self, key: &str, default: std::time::Duration) -> std::time::Duration {
		self.get_duration(key).unwrap_or(default)
	}

	pub fn get_size_bytes(&self, key: &str) -> Option<u64> {
		self.values.get(key).and_then(|v| parse_size(v))
	}

	pub fn get_size_bytes_or(&self, key: &str, default: u64) -> u64 {
		self.get_size_bytes(key).unwrap_or(default)
	}

	pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
		self.values.insert(key.into(), value.into());
	}

	pub fn set_default(&mut self, key: impl Into<String>, value: impl Into<String>) {
		self.values.entry(key.into()).or_insert(value.into());
	}

	pub fn remove(&mut self, key: &str) -> Option<String> {
		self.values.remove(key)
	}

	pub fn has(&self, key: &str) -> bool {
		self.values.contains_key(key)
	}

	pub fn validate_required(&self, keys: &[&str]) -> MurConfigResult<()> {
		let missing: Vec<_> = keys.iter().filter(|k| !self.has(k)).collect();

		if missing.is_empty() {
			Ok(())
		} else if missing.len() == 1 {
			Err(MurConfigError::MissingKey(missing[0].to_string()))
		} else {
			Err(MurConfigError::Multiple(
				missing
					.into_iter()
					.map(|k| MurConfigError::MissingKey(k.to_string()))
					.collect(),
			))
		}
	}

	pub fn mark_secret(&mut self, key: &str) {
		if !self.metadata.secret_keys.contains(&key.to_string()) {
			self.metadata.secret_keys.push(key.to_string());
		}
	}

	pub fn is_secret(&self, key: &str) -> bool {
		self.metadata.secret_keys.contains(&key.to_string())
	}

	pub fn loaded_files(&self) -> &[String] {
		&self.metadata.loaded_files
	}

	pub fn environment(&self) -> Option<&String> {
		self.metadata.environment.as_ref()
	}

	pub fn keys(&self) -> impl Iterator<Item = &String> {
		self.values.keys()
	}

	pub fn len(&self) -> usize {
		self.values.len()
	}

	pub fn is_empty(&self) -> bool {
		self.values.is_empty()
	}

	pub fn into_arc(self) -> Arc<Self> {
		Arc::new(self)
	}

	pub fn subset(&self, prefix: &str) -> MurConfig {
		let values: HashMap<String, String> = self
			.values
			.iter()
			.filter_map(|(k, v)| {
				k.strip_prefix(prefix)
					.map(|stripped| (stripped.to_string(), v.clone()))
			})
			.collect();

		MurConfig {
			values,
			metadata: ConfigMetadata::default(),
		}
	}
}

impl Default for MurConfig {
	fn default() -> Self {
		Self::new()
	}
}
