use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MurApiSchema {
	#[serde(rename = "type", skip_serializing_if = "Option::is_none")]
	pub schema_type: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub format: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub default: Option<serde_json::Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub example: Option<serde_json::Value>,
	#[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
	pub enum_values: Option<Vec<serde_json::Value>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub items: Option<Box<MurApiSchema>>,
	#[serde(skip_serializing_if = "IndexMap::is_empty", default)]
	pub properties: IndexMap<String, MurApiSchema>,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub required: Vec<String>,
	#[serde(
		rename = "additionalProperties",
		skip_serializing_if = "Option::is_none"
	)]
	pub additional_properties: Option<Box<MurApiSchema>>,
	#[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
	pub reference: Option<String>,
	#[serde(rename = "allOf", skip_serializing_if = "Vec::is_empty", default)]
	pub all_of: Vec<MurApiSchema>,
	#[serde(rename = "oneOf", skip_serializing_if = "Vec::is_empty", default)]
	pub one_of: Vec<MurApiSchema>,
	#[serde(rename = "anyOf", skip_serializing_if = "Vec::is_empty", default)]
	pub any_of: Vec<MurApiSchema>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub nullable: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub minimum: Option<f64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub maximum: Option<f64>,
	#[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
	pub min_length: Option<u64>,
	#[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
	pub max_length: Option<u64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub pattern: Option<String>,
}

impl MurApiSchema {
	pub fn string() -> Self {
		Self {
			schema_type: Some("string".to_string()),
			..Default::default()
		}
	}

	pub fn integer() -> Self {
		Self {
			schema_type: Some("integer".to_string()),
			format: Some("int64".to_string()),
			..Default::default()
		}
	}

	pub fn number() -> Self {
		Self {
			schema_type: Some("number".to_string()),
			format: Some("double".to_string()),
			..Default::default()
		}
	}

	pub fn boolean() -> Self {
		Self {
			schema_type: Some("boolean".to_string()),
			..Default::default()
		}
	}

	pub fn array(items: MurApiSchema) -> Self {
		Self {
			schema_type: Some("array".to_string()),
			items: Some(Box::new(items)),
			..Default::default()
		}
	}

	pub fn object() -> Self {
		Self {
			schema_type: Some("object".to_string()),
			..Default::default()
		}
	}

	pub fn reference(name: impl Into<String>) -> Self {
		Self {
			reference: Some(format!("#/components/schemas/{}", name.into())),
			..Default::default()
		}
	}

	pub fn description(mut self, desc: impl Into<String>) -> Self {
		self.description = Some(desc.into());
		self
	}

	pub fn format(mut self, fmt: impl Into<String>) -> Self {
		self.format = Some(fmt.into());
		self
	}

	pub fn example(mut self, ex: impl Into<serde_json::Value>) -> Self {
		self.example = Some(ex.into());
		self
	}

	pub fn property(mut self, name: impl Into<String>, schema: MurApiSchema) -> Self {
		self.properties.insert(name.into(), schema);
		self
	}

	pub fn required_property(mut self, name: impl Into<String>, schema: MurApiSchema) -> Self {
		let name = name.into();
		self.required.push(name.clone());
		self.properties.insert(name, schema);
		self
	}

	pub fn nullable(mut self) -> Self {
		self.nullable = Some(true);
		self
	}

	pub fn enum_values<I, V>(mut self, values: I) -> Self
	where
		I: IntoIterator<Item = V>,
		V: Into<serde_json::Value>,
	{
		self.enum_values = Some(values.into_iter().map(|v| v.into()).collect());
		self
	}

	pub fn min(mut self, min: f64) -> Self {
		self.minimum = Some(min);
		self
	}

	pub fn max(mut self, max: f64) -> Self {
		self.maximum = Some(max);
		self
	}

	pub fn min_length(mut self, len: u64) -> Self {
		self.min_length = Some(len);
		self
	}

	pub fn max_length(mut self, len: u64) -> Self {
		self.max_length = Some(len);
		self
	}

	pub fn pattern(mut self, pat: impl Into<String>) -> Self {
		self.pattern = Some(pat.into());
		self
	}
}
