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

pub fn mur_load_config() -> MurConfig {
	MurConfig::from_env()
}

pub fn mur_load_config_required(required_keys: &[&str]) -> MurConfigResult<MurConfig> {
	let config = MurConfig::from_env();
	config.validate_required(required_keys)?;
	Ok(config)
}

pub fn mur_env(key: &'static str) -> Result<String, MurError> {
	std::env::var(key)
		.ok()
		.or_else(|| dotenv().get(key).cloned())
		.ok_or(MurError::NoEnv(key))
}

pub fn mur_env_or(key: &str, default: &str) -> String {
	std::env::var(key)
		.ok()
		.or_else(|| dotenv().get(key).cloned())
		.unwrap_or_else(|| default.to_string())
}

pub fn mur_env_parse<T: std::str::FromStr>(key: &str) -> Option<T> {
	std::env::var(key)
		.ok()
		.or_else(|| dotenv().get(key).cloned())
		.and_then(|v| v.parse().ok())
}

pub fn mur_env_parse_or<T: std::str::FromStr>(key: &str, default: T) -> T {
	mur_env_parse(key).unwrap_or(default)
}

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
