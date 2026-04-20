use std::fmt;

/// Error type for configuration loading and validation failures.
///
/// Returned by [`MurConfig`](crate::MurConfig) and [`MurConfigBuilder`](crate::MurConfigBuilder)
/// operations when a required key is missing, a value cannot be parsed, or a
/// configuration file cannot be read.
#[derive(Debug, Clone)]
pub enum MurConfigError {
	/// A required configuration key was not found in any source.
	MissingKey(String),
	/// A configuration value could not be parsed into the target type.
	ParseError { key: String, message: String },
	/// A configuration file could not be opened or read.
	FileError { path: String, message: String },
	/// A custom validation rule was violated.
	ValidationError(String),
	/// Multiple configuration errors aggregated into a single value.
	Multiple(Vec<MurConfigError>),
}

impl fmt::Display for MurConfigError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			MurConfigError::MissingKey(key) => {
				write!(f, "Missing required configuration key: {}", key)
			}
			MurConfigError::ParseError { key, message } => {
				write!(f, "Failed to parse configuration '{}': {}", key, message)
			}
			MurConfigError::FileError { path, message } => {
				write!(
					f,
					"Failed to read configuration file '{}': {}",
					path, message
				)
			}
			MurConfigError::ValidationError(msg) => {
				write!(f, "Configuration validation failed: {}", msg)
			}
			MurConfigError::Multiple(errors) => {
				writeln!(f, "Multiple configuration errors:")?;
				for (i, err) in errors.iter().enumerate() {
					writeln!(f, "  {}. {}", i + 1, err)?;
				}
				Ok(())
			}
		}
	}
}

impl std::error::Error for MurConfigError {}
