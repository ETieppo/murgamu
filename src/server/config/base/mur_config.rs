use super::MurConfigBuilder;
use super::MurConfigError;
use super::MurConfigResult;
use super::parse_duration;
use super::parse_size;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// The central configuration store for a Murgamu application.
///
/// `MurConfig` aggregates values from OS environment variables, `.env` files,
/// and programmatic overrides into a single `HashMap<String, String>`. It
/// provides typed accessors for common value types (strings, booleans, numbers,
/// durations, byte sizes) and supports marking keys as secrets for safe
/// debug output.
///
/// # Quick start
///
/// ```rust,ignore
/// let config = MurConfig::from_env();
/// let db_url = config.get_required("DATABASE_URL")?;
/// let port: u16 = config.get_or("PORT", 3000);
/// ```
///
/// For more control, use the builder:
///
/// ```rust,ignore
/// let config = MurConfig::builder()
///     .env_file(".env")
///     .required("DATABASE_URL")
///     .build()?;
/// ```
#[derive(Debug, Clone)]
pub struct MurConfig {
	/// All resolved configuration values.
	pub values: HashMap<String, String>,
	metadata: ConfigMetadata,
}

/// Internal metadata tracked alongside configuration values.
#[derive(Debug, Clone, Default)]
pub struct ConfigMetadata {
	/// Paths of every file that was successfully loaded.
	pub loaded_files: Vec<String>,
	/// Keys whose values should be hidden in debug output.
	pub secret_keys: Vec<String>,
	/// The detected environment name (e.g. `"production"`).
	pub environment: Option<String>,
}

impl MurConfig {
	/// Creates an empty configuration with no values.
	pub fn new() -> Self {
		Self {
			values: HashMap::new(),
			metadata: ConfigMetadata::default(),
		}
	}

	/// Creates a configuration pre-populated from a `HashMap`.
	pub fn from_map(values: HashMap<String, String>) -> Self {
		Self {
			values,
			metadata: ConfigMetadata::default(),
		}
	}

	/// Creates a configuration by loading OS environment variables and any
	/// `.env` files found on disk.
	///
	/// File loading order: `.env` → `.env.{profile}` → `.env.local`.
	/// OS environment variables always take the highest priority.
	pub fn from_env() -> Self {
		let mut config = Self::new();
		config.load_env();
		config.try_load_dotenv();
		config
	}

	/// Returns a [`MurConfigBuilder`] for fluent configuration assembly.
	pub fn builder() -> MurConfigBuilder {
		MurConfigBuilder::new()
	}

	/// Loads all current OS environment variables into the value store.
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

	/// Reads and parses a `.env`-format (or JSON) file, merging its values in.
	///
	/// Returns an error if the file cannot be read.
	pub fn load_file(&mut self, path: &str) -> MurConfigResult<()> {
		let content = std::fs::read_to_string(path).map_err(|e| MurConfigError::FileError {
			path: path.to_string(),
			message: e.to_string(),
		})?;

		self.parse_dotenv(&content);
		self.metadata.loaded_files.push(path.to_string());
		Ok(())
	}

	/// Parses a `.env`-format string and inserts all key/value pairs.
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

	/// Returns the value for `key`, or `None` if it is not set.
	pub fn get(&self, key: &str) -> Option<&String> {
		self.values.get(key)
	}

	/// Returns the value for `key` as a `String`, falling back to `default`.
	pub fn get_or_string(&self, key: &str, default: &str) -> String {
		self.values
			.get(key)
			.cloned()
			.unwrap_or_else(|| default.to_string())
	}

	/// Returns the value for `key` parsed as `T`, falling back to `default`.
	pub fn get_or<T: std::str::FromStr>(&self, key: &str, default: T) -> T {
		self.values
			.get(key)
			.and_then(|v| v.parse().ok())
			.unwrap_or(default)
	}

	/// Returns the value for `key`, returning an error if it is absent.
	pub fn get_required(&self, key: &str) -> MurConfigResult<String> {
		self.values
			.get(key)
			.cloned()
			.ok_or_else(|| MurConfigError::MissingKey(key.to_string()))
	}

	/// Returns the value for `key` parsed as `T`, returning an error if it is
	/// absent or cannot be parsed.
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

	/// Returns the value for `key` interpreted as a boolean.
	///
	/// The strings `"1"`, `"true"`, `"yes"`, and `"on"` are truthy (case-insensitive).
	pub fn get_bool(&self, key: &str) -> Option<bool> {
		self.values.get(key).map(|v| {
			let v = v.to_lowercase();
			v == "1" || v == "true" || v == "yes" || v == "on"
		})
	}

	/// Returns the boolean value for `key`, falling back to `default`.
	pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
		self.get_bool(key).unwrap_or(default)
	}

	/// Returns the value for `key` split on commas, trimming whitespace from each item.
	pub fn get_list(&self, key: &str) -> Option<Vec<String>> {
		self.values.get(key).map(|v| {
			v.split(',')
				.map(|s| s.trim().to_string())
				.filter(|s| !s.is_empty())
				.collect()
		})
	}

	/// Returns the comma-separated list for `key`, falling back to `default`.
	pub fn get_list_or(&self, key: &str, default: Vec<String>) -> Vec<String> {
		self.get_list(key).unwrap_or(default)
	}

	/// Parses the value for `key` as a duration using [`parse_duration`].
	pub fn get_duration(&self, key: &str) -> Option<std::time::Duration> {
		self.values.get(key).and_then(|v| parse_duration(v))
	}

	/// Returns the parsed duration for `key`, falling back to `default`.
	pub fn get_duration_or(&self, key: &str, default: std::time::Duration) -> std::time::Duration {
		self.get_duration(key).unwrap_or(default)
	}

	/// Parses the value for `key` as a byte size using [`parse_size`].
	pub fn get_size_bytes(&self, key: &str) -> Option<u64> {
		self.values.get(key).and_then(|v| parse_size(v))
	}

	/// Returns the parsed byte size for `key`, falling back to `default`.
	pub fn get_size_bytes_or(&self, key: &str, default: u64) -> u64 {
		self.get_size_bytes(key).unwrap_or(default)
	}

	/// Inserts or overwrites `key` with `value`.
	pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
		self.values.insert(key.into(), value.into());
	}

	/// Sets `key` to `value` only if `key` is not already present.
	pub fn set_default(&mut self, key: impl Into<String>, value: impl Into<String>) {
		self.values.entry(key.into()).or_insert(value.into());
	}

	/// Removes `key` from the store, returning its previous value if any.
	pub fn remove(&mut self, key: &str) -> Option<String> {
		self.values.remove(key)
	}

	/// Returns `true` if `key` exists in the store.
	pub fn has(&self, key: &str) -> bool {
		self.values.contains_key(key)
	}

	/// Returns an error listing every key in `keys` that is not present.
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

	/// Marks `key` as a secret so its value is hidden in debug representations.
	pub fn mark_secret(&mut self, key: &str) {
		if !self.metadata.secret_keys.contains(&key.to_string()) {
			self.metadata.secret_keys.push(key.to_string());
		}
	}

	/// Returns `true` if `key` has been marked as a secret.
	pub fn is_secret(&self, key: &str) -> bool {
		self.metadata.secret_keys.contains(&key.to_string())
	}

	/// Returns the list of files that were successfully loaded.
	pub fn loaded_files(&self) -> &[String] {
		&self.metadata.loaded_files
	}

	/// Returns the detected environment name, if any.
	pub fn environment(&self) -> Option<&String> {
		self.metadata.environment.as_ref()
	}

	/// Returns an iterator over all configuration keys.
	pub fn keys(&self) -> impl Iterator<Item = &String> {
		self.values.keys()
	}

	/// Returns the total number of configuration entries.
	pub fn len(&self) -> usize {
		self.values.len()
	}

	/// Returns `true` if the configuration store is empty.
	pub fn is_empty(&self) -> bool {
		self.values.is_empty()
	}

	/// Wraps `self` in an [`Arc`] for shared ownership.
	pub fn into_arc(self) -> Arc<Self> {
		Arc::new(self)
	}

	/// Returns a new `MurConfig` containing only the keys that start with `prefix`,
	/// with the prefix stripped from each key name.
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
