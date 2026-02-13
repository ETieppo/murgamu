use std::fmt;

#[derive(Debug, Clone)]
pub enum MurConfigError {
	MissingKey(String),
	ParseError { key: String, message: String },
	FileError { path: String, message: String },
	ValidationError(String),
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
