pub struct MurEnv;
use super::MurEnvProfile;

impl MurEnv {
	pub fn current() -> MurEnvProfile {
		let env_str = std::env::var("MUR_ENV")
			.or_else(|_| std::env::var("APP_ENV"))
			.or_else(|_| std::env::var("RUST_ENV"))
			.or_else(|_| std::env::var("NODE_ENV"))
			.unwrap_or_else(|_| "development".to_string());

		MurEnvProfile::from_str(&env_str)
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
		std::env::set_var("MUR_ENV", profile.as_str());
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
