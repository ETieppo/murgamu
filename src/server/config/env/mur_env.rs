pub struct MurEnv;
use super::MurEnvProfile;

impl MurEnv {
	pub fn current() -> MurEnvProfile {
		let env_str = std::env::var("MUR_ENV")
			.or_else(|_| std::env::var("APP_ENV"))
			.or_else(|_| std::env::var("RUST_ENV"))
			.or_else(|_| std::env::var("NODE_ENV"))
			.unwrap_or_else(|_| "development".to_string());

		MurEnvProfile::from_name(&env_str)
	}

	/// Loads `.env`, `.env.{profile}` and `.env.local` into the process environment.
	///
	/// Values already present in the OS environment are NOT overwritten, so real
	/// env-vars always take priority over the files.
	///
	/// # Safety
	/// Mutates process-wide environment variables. Must be called before any
	/// other threads are spawned (e.g. at the very start of `main`).
	pub unsafe fn load() {
		for file in Self::env_files() {
			unsafe { Self::load_file_into_env(&file) };
		}
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

	/// Sets the current Mur environment profile.
	///
	/// # Safety
	/// This function mutates process environment variables.
	/// The caller must guarantee that no other threads are concurrently
	/// reading or writing environment variables while this function runs.
	/// Typically this means it must be called during program initialization
	/// before any threads are spawned.
	pub unsafe fn set(profile: MurEnvProfile) {
		unsafe { std::env::set_var("MUR_ENV", profile.as_str()) };
	}

	pub fn is_development() -> bool {
		Self::current().is_development()
	}

	pub fn is_production() -> bool {
		Self::current().is_production()
	}

	pub fn is_staging() -> bool {
		Self::current().is_staging()
	}

	pub fn is_test() -> bool {
		Self::current().is_test()
	}

	pub fn env_file() -> String {
		Self::current().env_file()
	}

	pub fn env_files() -> Vec<String> {
		Self::current().env_files()
	}
}
