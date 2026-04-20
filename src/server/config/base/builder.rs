use super::MurConfig;
use super::MurConfigResult;
use std::collections::HashMap;
use std::path::Path;

/// Fluent builder for constructing a [`MurConfig`] from multiple sources.
///
/// `MurConfigBuilder` lets you compose configuration from `.env` files,
/// OS environment variables, compile-time defaults, and programmatic
/// overrides — in a predictable, explicit priority order.
///
/// Priority (highest → lowest):
/// 1. Values set via [`set`](Self::set)
/// 2. OS environment variables (when [`no_env`](Self::no_env) is not called)
/// 3. `.env` files added via [`env_file`](Self::env_file), last added wins
/// 4. Defaults registered via [`default`](Self::default)
///
/// # Example
///
/// ```rust,ignore
/// let config = MurConfig::builder()
///     .env_file(".env")
///     .env_file_for_env(".env.production")
///     .required("DATABASE_URL")
///     .default("PORT", "3000")
///     .secret("DATABASE_URL")
///     .build()?;
/// ```
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
	/// Creates a builder with OS environment loading enabled.
	pub fn new() -> Self {
		Self {
			load_env: true,
			..Default::default()
		}
	}

	/// Adds a `.env`-format file to the source list.
	///
	/// The file must exist; the build step returns an error otherwise.
	pub fn env_file(mut self, path: &str) -> Self {
		self.env_files.push(path.to_string());
		self
	}

	/// Adds a `.env`-format file only if it exists on disk.
	///
	/// Useful for profile-specific files (e.g. `.env.production`) that may
	/// not be present in every environment.
	pub fn env_file_for_env(mut self, path: &str) -> Self {
		if Path::new(path).exists() {
			self.env_files.push(path.to_string());
		}
		self
	}

	/// Marks `key` as required.
	///
	/// [`build`](Self::build) returns [`MurConfigError::MissingKey`](crate::MurConfigError) if
	/// the key is not present in any source.
	pub fn required(mut self, key: &str) -> Self {
		self.required_keys.push(key.to_string());
		self
	}

	/// Sets a fallback value for `key` used when no other source provides it.
	pub fn default(mut self, key: &str, value: &str) -> Self {
		self.defaults.insert(key.to_string(), value.to_string());
		self
	}

	/// Marks `key` as a secret so it is redacted from debug output.
	pub fn secret(mut self, key: &str) -> Self {
		self.secrets.push(key.to_string());
		self
	}

	/// Programmatically sets `key` to `value`, overriding all other sources.
	pub fn set(mut self, key: &str, value: &str) -> Self {
		self.values.insert(key.to_string(), value.to_string());
		self
	}

	/// Disables loading of OS environment variables.
	pub fn no_env(mut self) -> Self {
		self.load_env = false;
		self
	}

	/// Builds the [`MurConfig`], validating required keys and applying all sources.
	///
	/// Returns [`Err`] if any required key is missing or a file cannot be read.
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
