use super::EnvServerDefaults;
use std::fmt;

/// The active deployment environment of the application.
///
/// `MurEnvProfile` is resolved from the first environment variable found in
/// the sequence `MUR_ENV` → `APP_ENV` → `RUST_ENV` → `NODE_ENV`. If none is
/// set, [`Development`](MurEnvProfile::Development) is assumed.
///
/// Each profile provides sensible defaults via
/// [`server_defaults`](MurEnvProfile::server_defaults) and controls which
/// `.env` files are loaded.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum MurEnvProfile {
	/// Staging / pre-production environment.
	Staging,
	/// Live production environment.
	Production,
	/// Automated test / CI environment.
	Test,
	/// A non-standard environment identified by name.
	Custom(String),
	/// Local development environment (default).
	#[default]
	Development,
}

impl MurEnvProfile {
	/// Parses an environment name string into the corresponding profile.
	///
	/// Recognised aliases:
	/// - `"development"` | `"dev"` | `"local"` → [`Development`](Self::Development)
	/// - `"staging"` | `"stage"` | `"stg"` → [`Staging`](Self::Staging)
	/// - `"production"` | `"prod"` | `"live"` → [`Production`](Self::Production)
	/// - `"test"` | `"testing"` | `"ci"` → [`Test`](Self::Test)
	/// - anything else → [`Custom(name)`](Self::Custom)
	pub fn from_name(s: &str) -> Self {
		match s.to_lowercase().as_str() {
			"development" | "dev" | "local" => MurEnvProfile::Development,
			"staging" | "stage" | "stg" => MurEnvProfile::Staging,
			"production" | "prod" | "live" => MurEnvProfile::Production,
			"test" | "testing" | "ci" => MurEnvProfile::Test,
			other => MurEnvProfile::Custom(other.to_string()),
		}
	}

	/// Returns the canonical lowercase name of the profile.
	pub fn as_str(&self) -> &str {
		match self {
			MurEnvProfile::Development => "development",
			MurEnvProfile::Staging => "staging",
			MurEnvProfile::Production => "production",
			MurEnvProfile::Test => "test",
			MurEnvProfile::Custom(s) => s,
		}
	}

	/// Returns `true` if this is the [`Development`](Self::Development) profile.
	pub fn is_development(&self) -> bool {
		matches!(self, MurEnvProfile::Development)
	}

	/// Returns `true` if this is the [`Staging`](Self::Staging) profile.
	pub fn is_staging(&self) -> bool {
		matches!(self, MurEnvProfile::Staging)
	}

	/// Returns `true` if this is the [`Production`](Self::Production) profile.
	pub fn is_production(&self) -> bool {
		matches!(self, MurEnvProfile::Production)
	}

	/// Returns `true` if this is the [`Test`](Self::Test) profile.
	pub fn is_test(&self) -> bool {
		matches!(self, MurEnvProfile::Test)
	}

	/// Returns `true` if this is a [`Custom`](Self::Custom) profile.
	pub fn is_custom(&self) -> bool {
		matches!(self, MurEnvProfile::Custom(_))
	}

	/// Returns `true` for profiles that behave like production
	/// ([`Production`](Self::Production) and [`Staging`](Self::Staging)).
	pub fn is_production_like(&self) -> bool {
		matches!(self, MurEnvProfile::Production | MurEnvProfile::Staging)
	}

	/// Returns `true` for profiles that behave like development
	/// ([`Development`](Self::Development) and [`Test`](Self::Test)).
	pub fn is_development_like(&self) -> bool {
		matches!(self, MurEnvProfile::Development | MurEnvProfile::Test)
	}

	/// Returns the path of the profile-specific `.env` file (e.g. `.env.production`).
	pub fn env_file(&self) -> String {
		format!(".env.{}", self.as_str())
	}

	/// Returns the ordered list of `.env` file paths to load for this profile.
	///
	/// Order: `.env` → `.env.{profile}` → `.env.local`.
	/// Later files take precedence over earlier ones.
	pub fn env_files(&self) -> Vec<String> {
		vec![
			".env".to_string(),
			self.env_file(),
			".env.local".to_string(),
		]
	}

	/// Returns the recommended log level for this profile.
	pub fn default_log_level(&self) -> &str {
		match self {
			MurEnvProfile::Development => "debug",
			MurEnvProfile::Staging => "info",
			MurEnvProfile::Production => "warn",
			MurEnvProfile::Test => "error",
			MurEnvProfile::Custom(_) => "info",
		}
	}

	/// Returns `true` if verbose output should be enabled for this profile.
	pub fn default_verbose(&self) -> bool {
		matches!(self, MurEnvProfile::Development)
	}

	/// Returns `true` if debug features are enabled for this profile.
	pub fn debug_enabled(&self) -> bool {
		self.is_development_like()
	}

	/// Returns the human-readable display name of the profile.
	pub fn display_name(&self) -> &str {
		match self {
			MurEnvProfile::Development => "Development",
			MurEnvProfile::Staging => "Staging",
			MurEnvProfile::Production => "Production",
			MurEnvProfile::Test => "Test",
			MurEnvProfile::Custom(s) => s,
		}
	}

	/// Returns a representative emoji for the profile (useful in startup banners).
	pub fn emoji(&self) -> &str {
		match self {
			MurEnvProfile::Development => "🔧",
			MurEnvProfile::Staging => "🎭",
			MurEnvProfile::Production => "🚀",
			MurEnvProfile::Test => "🧪",
			MurEnvProfile::Custom(_) => "📦",
		}
	}

	/// Returns the default server configuration for this profile.
	pub fn server_defaults(&self) -> EnvServerDefaults {
		match self {
			MurEnvProfile::Development => EnvServerDefaults {
				host: "127.0.0.1".to_string(),
				port: 3000,
				enable_cors: true,
				enable_logging: true,
				log_requests: true,
				graceful_shutdown: false,
				keep_alive_secs: 60,
			},
			MurEnvProfile::Staging => EnvServerDefaults {
				host: "0.0.0.0".to_string(),
				port: 3000,
				enable_cors: true,
				enable_logging: true,
				log_requests: true,
				graceful_shutdown: true,
				keep_alive_secs: 75,
			},
			MurEnvProfile::Production => EnvServerDefaults {
				host: "0.0.0.0".to_string(),
				port: 8080,
				enable_cors: false,
				enable_logging: true,
				log_requests: false,
				graceful_shutdown: true,
				keep_alive_secs: 75,
			},
			MurEnvProfile::Test => EnvServerDefaults {
				host: "127.0.0.1".to_string(),
				port: 0,
				enable_cors: true,
				enable_logging: false,
				log_requests: false,
				graceful_shutdown: false,
				keep_alive_secs: 5,
			},
			MurEnvProfile::Custom(_) => EnvServerDefaults {
				host: "127.0.0.1".to_string(),
				port: 3000,
				enable_cors: false,
				enable_logging: true,
				log_requests: false,
				graceful_shutdown: true,
				keep_alive_secs: 60,
			},
		}
	}
}

impl fmt::Display for MurEnvProfile {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl From<&str> for MurEnvProfile {
	fn from(s: &str) -> Self {
		MurEnvProfile::from_name(s)
	}
}

impl From<String> for MurEnvProfile {
	fn from(s: String) -> Self {
		MurEnvProfile::from_name(&s)
	}
}
