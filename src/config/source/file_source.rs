use super::MurConfigSource;
use crate::config::{MurConfigError, MurConfigResult};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
	DotEnv,
	Json,
	Toml,
}

#[derive(Debug, Clone)]
pub struct MurFileSource {
	path: String,
	required: bool,
	format: Option<FileFormat>,
}

impl MurFileSource {
	pub fn new(path: &str) -> Self {
		Self {
			path: path.to_string(),
			required: true,
			format: None,
		}
	}

	pub fn optional(path: &str) -> Self {
		Self {
			path: path.to_string(),
			required: false,
			format: None,
		}
	}

	pub fn required(mut self, required: bool) -> Self {
		self.required = required;
		self
	}

	pub fn format(mut self, format: FileFormat) -> Self {
		self.format = Some(format);
		self
	}

	fn detect_format(&self) -> FileFormat {
		if let Some(format) = self.format {
			return format;
		}

		let path = Path::new(&self.path);
		match path.extension().and_then(|e| e.to_str()) {
			Some("json") => FileFormat::Json,
			Some("toml") => FileFormat::Toml,
			_ => FileFormat::DotEnv,
		}
	}

	pub fn parse_dotenv(content: &str) -> HashMap<String, String> {
		let mut values = HashMap::new();

		for line in content.lines() {
			let line = line.trim();

			if line.is_empty() || line.starts_with('#') {
				continue;
			}

			if let Some((key, value)) = line.split_once('=') {
				let key = key.trim().to_string();
				let value = Self::parse_value(value.trim());
				values.insert(key, value);
			}
		}

		values
	}

	fn parse_value(value: &str) -> String {
		let value = value.trim();

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

	pub fn parse_json(content: &str) -> MurConfigResult<HashMap<String, String>> {
		let json: serde_json::Value =
			serde_json::from_str(content).map_err(|e| MurConfigError::ParseError {
				key: "JSON".to_string(),
				message: e.to_string(),
			})?;

		let mut values = HashMap::new();
		Self::flatten_json("", &json, &mut values);
		Ok(values)
	}

	fn flatten_json(prefix: &str, value: &serde_json::Value, map: &mut HashMap<String, String>) {
		match value {
			serde_json::Value::Object(obj) => {
				for (key, val) in obj {
					let new_prefix = if prefix.is_empty() {
						key.to_uppercase()
					} else {
						format!("{}_{}", prefix, key.to_uppercase())
					};
					Self::flatten_json(&new_prefix, val, map);
				}
			}
			serde_json::Value::Array(arr) => {
				for (i, val) in arr.iter().enumerate() {
					let new_prefix = format!("{}_{}", prefix, i);
					Self::flatten_json(&new_prefix, val, map);
				}
			}
			serde_json::Value::String(s) => {
				map.insert(prefix.to_string(), s.clone());
			}
			serde_json::Value::Number(n) => {
				map.insert(prefix.to_string(), n.to_string());
			}
			serde_json::Value::Bool(b) => {
				map.insert(prefix.to_string(), b.to_string());
			}
			serde_json::Value::Null => {
				map.insert(prefix.to_string(), String::new());
			}
		}
	}
}

impl MurConfigSource for MurFileSource {
	fn load(&self) -> MurConfigResult<HashMap<String, String>> {
		if !Path::new(&self.path).exists() {
			if self.required {
				return Err(MurConfigError::FileError {
					path: self.path.clone(),
					message: "File not found".to_string(),
				});
			}
			return Ok(HashMap::new());
		}

		let content =
			std::fs::read_to_string(&self.path).map_err(|e| MurConfigError::FileError {
				path: self.path.clone(),
				message: e.to_string(),
			})?;

		match self.detect_format() {
			FileFormat::DotEnv => Ok(Self::parse_dotenv(&content)),
			FileFormat::Json => Self::parse_json(&content),
			FileFormat::Toml => Err(MurConfigError::ParseError {
				key: "TOML".to_string(),
				message: "TOML format not yet supported. Use .env or JSON instead.".to_string(),
			}),
		}
	}

	fn name(&self) -> &str {
		&self.path
	}

	fn priority(&self) -> i32 {
		10
	}

	fn is_available(&self) -> bool {
		!self.required || Path::new(&self.path).exists()
	}
}
