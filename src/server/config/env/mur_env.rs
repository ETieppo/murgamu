pub struct MurEnv;
use super::MurEnvProfile;
use std::sync::OnceLock;

static ENV_LOADED: OnceLock<()> = OnceLock::new();

/// Utilities for reading and loading the application's runtime environment.
///
/// `MurEnv` resolves the active [`MurEnvProfile`] from standard environment
/// variables and provides methods for loading `.env` files into the process
/// environment so that every call to [`mur_env`](crate::mur_env) or
/// [`std::env::var`] can find them.
///
/// # Environment variable resolution order
///
/// `MUR_ENV` → `APP_ENV` → `RUST_ENV` → `NODE_ENV` → `"development"`
///
/// # `.env` file loading order
///
/// `.env` → `.env.{profile}` → `.env.local`
///
/// Later files take precedence over earlier ones. OS environment variables
/// are never overwritten — they always have the highest priority.
impl MurEnv {
	/// Detects and returns the active [`MurEnvProfile`].
	pub fn current() -> MurEnvProfile {
		let env_str = std::env::var("MUR_ENV")
			.or_else(|_| std::env::var("APP_ENV"))
			.or_else(|_| std::env::var("RUST_ENV"))
			.or_else(|_| std::env::var("NODE_ENV"))
			.unwrap_or_else(|_| "development".to_string());

		MurEnvProfile::from_name(&env_str)
	}

	/// Loads `.env`, `.env.{profile}`, and `.env.local` into the process environment.
	///
	/// Keys already present in the OS environment are **not** overwritten, so
	/// real environment variables always take priority over file contents.
	///
	/// # Safety
	///
	/// Mutates process-wide environment variables via [`std::env::set_var`].
	/// Must be called before any other threads are spawned — typically at the
	/// very top of `main` or inside [`MurServer::new`](crate::MurServer::new).
	pub unsafe fn load() {
		ENV_LOADED.get_or_init(|| {
			for file in Self::env_files() {
				// SAFETY: caller guarantees no other threads are running yet.
				unsafe { Self::load_file_into_env(&file) };
			}
		});
	}

	unsafe fn load_file_into_env(path: &str) {
		let Ok(content) = std::fs::read_to_string(path) else {
			return;
		};
		for line in content.lines() {
			let line = line.trim();
			if line.is_empty() || line.starts_with('#') {
				continue;
			}
			if let Some((key, raw_value)) = line.split_once('=') {
				let key = key.trim();
				if key.is_empty() || std::env::var(key).is_ok() {
					continue;
				}
				let value = Self::parse_dotenv_value(raw_value.trim());
				unsafe { std::env::set_var(key, value) };
			}
		}
	}

	fn parse_dotenv_value(value: &str) -> String {
		// A quoted value needs at least the two surrounding quotes. Guarding on
		// `len() >= 2` avoids a panic on a lone quote char (e.g. `KEY="`), where
		// `value[1..value.len() - 1]` would slice with start > end.
		if value.len() >= 2
			&& ((value.starts_with('"') && value.ends_with('"'))
				|| (value.starts_with('\'') && value.ends_with('\'')))
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

	/// Overrides the active environment profile.
	///
	/// # Safety
	///
	/// Mutates the process-wide `MUR_ENV` environment variable. Must be called
	/// before any other threads are spawned.
	pub unsafe fn set(profile: MurEnvProfile) {
		unsafe { std::env::set_var("MUR_ENV", profile.as_str()) };
	}

	/// Returns `true` if the active profile is [`Development`](MurEnvProfile::Development).
	pub fn is_development() -> bool {
		Self::current().is_development()
	}

	/// Returns `true` if the active profile is [`Production`](MurEnvProfile::Production).
	pub fn is_production() -> bool {
		Self::current().is_production()
	}

	/// Returns `true` if the active profile is [`Staging`](MurEnvProfile::Staging).
	pub fn is_staging() -> bool {
		Self::current().is_staging()
	}

	/// Returns `true` if the active profile is [`Test`](MurEnvProfile::Test).
	pub fn is_test() -> bool {
		Self::current().is_test()
	}

	/// Returns the path of the profile-specific `.env` file (e.g. `.env.production`).
	pub fn env_file() -> String {
		Self::current().env_file()
	}

	/// Returns the ordered list of `.env` files to load for the active profile.
	pub fn env_files() -> Vec<String> {
		Self::current().env_files()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_plain_value() {
		assert_eq!(MurEnv::parse_dotenv_value("plain"), "plain");
	}

	#[test]
	fn parses_double_quoted_with_escapes() {
		assert_eq!(MurEnv::parse_dotenv_value(r#""a\nb""#), "a\nb");
		assert_eq!(MurEnv::parse_dotenv_value(r#""tab\tend""#), "tab\tend");
	}

	#[test]
	fn parses_single_quoted_literally() {
		assert_eq!(MurEnv::parse_dotenv_value(r#"'a\nb'"#), r#"a\nb"#);
	}

	#[test]
	fn strips_trailing_inline_comment() {
		assert_eq!(MurEnv::parse_dotenv_value("value # comment"), "value");
	}

	#[test]
	fn empty_value_does_not_panic() {
		assert_eq!(MurEnv::parse_dotenv_value(""), "");
	}

	#[test]
	fn lone_quote_does_not_panic() {
		// Regression: a single quote char used to slice value[1..0] and panic.
		assert_eq!(MurEnv::parse_dotenv_value("\""), "\"");
		assert_eq!(MurEnv::parse_dotenv_value("'"), "'");
	}

	#[test]
	fn quote_only_pair_is_empty() {
		assert_eq!(MurEnv::parse_dotenv_value("\"\""), "");
		assert_eq!(MurEnv::parse_dotenv_value("''"), "");
	}
}
