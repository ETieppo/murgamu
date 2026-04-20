use super::MurConfig;
use super::MurConfigResult;
use super::MurEnv;
use super::MurEnvProfile;
use crate::server::error::MurError;
use std::collections::HashMap;
use std::sync::OnceLock;

static DOTENV: OnceLock<HashMap<String, String>> = OnceLock::new();

fn dotenv() -> &'static HashMap<String, String> {
	DOTENV.get_or_init(|| {
		let mut map = HashMap::new();
		let profile = std::env::var("MUR_ENV")
			.or_else(|_| std::env::var("APP_ENV"))
			.or_else(|_| std::env::var("RUST_ENV"))
			.or_else(|_| std::env::var("NODE_ENV"))
			.unwrap_or_else(|_| "development".to_string());

		let files = [
			".env".to_string(),
			format!(".env.{}", profile),
			".env.local".to_string(),
		];

		for file in &files {
			let Ok(content) = std::fs::read_to_string(file) else {
				continue;
			};
			for line in content.lines() {
				let line = line.trim();
				if line.is_empty() || line.starts_with('#') {
					continue;
				}
				if let Some((key, raw)) = line.split_once('=') {
					let key = key.trim().to_string();
					if !key.is_empty() {
						map.entry(key).or_insert_with(|| parse_dotenv_val(raw.trim()));
					}
				}
			}
		}
		map
	})
}

fn parse_dotenv_val(v: &str) -> String {
	if (v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')) {
		let inner = &v[1..v.len() - 1];
		if v.starts_with('"') {
			return inner
				.replace("\\n", "\n")
				.replace("\\t", "\t")
				.replace("\\r", "\r")
				.replace("\\\"", "\"")
				.replace("\\\\", "\\");
		}
		return inner.to_string();
	}
	if let Some(idx) = v.find(" #") {
		return v[..idx].trim().to_string();
	}
	v.to_string()
}

/// Loads all configuration sources and returns a [`MurConfig`] instance.
///
/// Equivalent to [`MurConfig::from_env`] — reads OS env vars and `.env` files.
pub fn mur_load_config() -> MurConfig {
	MurConfig::from_env()
}

/// Loads configuration and validates that `required_keys` are all present.
///
/// Returns an error listing every missing key.
pub fn mur_load_config_required(required_keys: &[&str]) -> MurConfigResult<MurConfig> {
	let config = MurConfig::from_env();
	config.validate_required(required_keys)?;
	Ok(config)
}

/// Returns the value of the environment variable `key`.
///
/// Checks the OS environment first; if the key is absent, falls back to a
/// lazily-loaded cache populated from `.env`, `.env.{profile}`, and
/// `.env.local`. This means `mur_env` works correctly even when called
/// before [`MurServer::new`](crate::MurServer::new).
///
/// Returns [`MurError::NoEnv`] if the key is not found in any source.
///
/// # Example
///
/// ```rust,ignore
/// let db_url = mur_env("DATABASE_URL")?;
/// ```
pub fn mur_env(key: &'static str) -> Result<String, MurError> {
	std::env::var(key)
		.ok()
		.or_else(|| dotenv().get(key).cloned())
		.ok_or(MurError::NoEnv(key))
}

/// Returns the value of `key`, or `default` if the key is absent.
pub fn mur_env_or(key: &str, default: &str) -> String {
	std::env::var(key)
		.ok()
		.or_else(|| dotenv().get(key).cloned())
		.unwrap_or_else(|| default.to_string())
}

/// Parses the value of `key` into `T`, returning `None` if absent or unparseable.
pub fn mur_env_parse<T: std::str::FromStr>(key: &str) -> Option<T> {
	std::env::var(key)
		.ok()
		.or_else(|| dotenv().get(key).cloned())
		.and_then(|v| v.parse().ok())
}

/// Parses the value of `key` into `T`, falling back to `default`.
pub fn mur_env_parse_or<T: std::str::FromStr>(key: &str, default: T) -> T {
	mur_env_parse(key).unwrap_or(default)
}

/// Returns `true` if the value of `key` is a truthy string.
///
/// Truthy values (case-insensitive): `"1"`, `"true"`, `"yes"`, `"on"`.
pub fn mur_env_is_true(key: &str) -> bool {
	std::env::var(key)
		.ok()
		.or_else(|| dotenv().get(key).cloned())
		.map(|v| {
			let v = v.to_lowercase();
			v == "1" || v == "true" || v == "yes" || v == "on"
		})
		.unwrap_or(false)
}

/// Returns the active [`MurEnvProfile`].
pub fn mur_current_env() -> MurEnvProfile {
	MurEnv::current()
}

/// Returns `true` if the active profile is [`Development`](MurEnvProfile::Development).
pub fn mur_is_development() -> bool {
	mur_current_env().is_development()
}

/// Returns `true` if the active profile is [`Production`](MurEnvProfile::Production).
pub fn mur_is_production() -> bool {
	mur_current_env().is_production()
}

/// Returns `true` if the active profile is [`Test`](MurEnvProfile::Test).
pub fn mur_is_test() -> bool {
	mur_current_env().is_test()
}
