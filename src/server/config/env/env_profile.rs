use super::EnvServerDefaults;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum MurEnvProfile {
	Staging,
	Production,
	Test,
	Custom(String),
	#[default]
	Development,
}

impl MurEnvProfile {
	pub fn from_name(s: &str) -> Self {
		match s.to_lowercase().as_str() {
			"development" | "dev" | "local" => MurEnvProfile::Development,
			"staging" | "stage" | "stg" => MurEnvProfile::Staging,
			"production" | "prod" | "live" => MurEnvProfile::Production,
			"test" | "testing" | "ci" => MurEnvProfile::Test,
			other => MurEnvProfile::Custom(other.to_string()),
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			MurEnvProfile::Development => "development",
			MurEnvProfile::Staging => "staging",
			MurEnvProfile::Production => "production",
			MurEnvProfile::Test => "test",
			MurEnvProfile::Custom(s) => s,
		}
	}

	pub fn is_development(&self) -> bool {
		matches!(self, MurEnvProfile::Development)
	}

	pub fn is_staging(&self) -> bool {
		matches!(self, MurEnvProfile::Staging)
	}

	pub fn is_production(&self) -> bool {
		matches!(self, MurEnvProfile::Production)
	}

	pub fn is_test(&self) -> bool {
		matches!(self, MurEnvProfile::Test)
	}

	pub fn is_custom(&self) -> bool {
		matches!(self, MurEnvProfile::Custom(_))
	}

	pub fn is_production_like(&self) -> bool {
		matches!(self, MurEnvProfile::Production | MurEnvProfile::Staging)
	}

	pub fn is_development_like(&self) -> bool {
		matches!(self, MurEnvProfile::Development | MurEnvProfile::Test)
	}

	pub fn env_file(&self) -> String {
		format!(".env.{}", self.as_str())
	}

	pub fn env_files(&self) -> Vec<String> {
		vec![
			".env".to_string(),
			self.env_file(),
			".env.local".to_string(),
		]
	}

	pub fn default_log_level(&self) -> &str {
		match self {
			MurEnvProfile::Development => "debug",
			MurEnvProfile::Staging => "info",
			MurEnvProfile::Production => "warn",
			MurEnvProfile::Test => "error",
			MurEnvProfile::Custom(_) => "info",
		}
	}

	pub fn default_verbose(&self) -> bool {
		matches!(self, MurEnvProfile::Development)
	}

	pub fn debug_enabled(&self) -> bool {
		self.is_development_like()
	}

	pub fn display_name(&self) -> &str {
		match self {
			MurEnvProfile::Development => "Development",
			MurEnvProfile::Staging => "Staging",
			MurEnvProfile::Production => "Production",
			MurEnvProfile::Test => "Test",
			MurEnvProfile::Custom(s) => s,
		}
	}

	pub fn emoji(&self) -> &str {
		match self {
			MurEnvProfile::Development => "🔧",
			MurEnvProfile::Staging => "🎭",
			MurEnvProfile::Production => "🚀",
			MurEnvProfile::Test => "🧪",
			MurEnvProfile::Custom(_) => "📦",
		}
	}

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
