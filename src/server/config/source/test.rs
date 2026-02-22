use super::*;

#[test]
fn test_memory_source() {
	let mut source = MurMemorySource::new();
	source.set("KEY1", "value1");
	source.set("KEY2", "value2");

	let values = source.load().unwrap();
	assert_eq!(values.get("KEY1"), Some(&"value1".to_string()));
	assert_eq!(values.get("KEY2"), Some(&"value2".to_string()));
}

// #[test]
// fn test_env_source() {
// 	std::env::set_var("MURGAMU_TEST_KEY", "test_value");

// 	let source = MurEnvSource::new();
// 	let values = source.load().unwrap();

// 	assert_eq!(
// 		values.get("MURGAMU_TEST_KEY"),
// 		Some(&"test_value".to_string())
// 	);

// 	std::env::remove_var("MURGAMU_TEST_KEY");
// }

// #[test]
// fn test_env_source_with_prefix() {
// 	std::env::set_var("TEST_PREFIX_KEY", "value");

// 	let source = MurEnvSource::with_prefix("TEST_PREFIX_");
// 	let values = source.load().unwrap();

// 	assert_eq!(values.get("KEY"), Some(&"value".to_string()));
// 	std::env::remove_var("TEST_PREFIX_KEY");
// }

#[test]
fn test_file_source_optional() {
	let source = MurFileSource::optional("nonexistent.env");
	let values = source.load().unwrap();
	assert!(values.is_empty());
}

#[test]
fn test_parse_dotenv() {
	let content = r#"
# Comment
KEY1=value1
KEY2="quoted value"
KEY3='single quoted'
KEY4=value with spaces
KEY5=value # inline comment
"#;

	let values = MurFileSource::parse_dotenv(content);
	assert_eq!(values.get("KEY1"), Some(&"value1".to_string()));
	assert_eq!(values.get("KEY2"), Some(&"quoted value".to_string()));
	assert_eq!(values.get("KEY3"), Some(&"single quoted".to_string()));
	assert_eq!(values.get("KEY4"), Some(&"value with spaces".to_string()));
	assert_eq!(values.get("KEY5"), Some(&"value".to_string()));
}

#[test]
fn test_chained_source() {
	let mut low_priority = MurMemorySource::new().with_priority(10);
	low_priority.set("KEY", "low");
	low_priority.set("LOW_ONLY", "value");

	let mut high_priority = MurMemorySource::new().with_priority(100);
	high_priority.set("KEY", "high");
	high_priority.set("HIGH_ONLY", "value");

	let chained = MurChainedSource::new()
		.add_source(low_priority)
		.add_source(high_priority);

	let values = chained.load().unwrap();
	assert_eq!(values.get("KEY"), Some(&"high".to_string()));
	assert_eq!(values.get("LOW_ONLY"), Some(&"value".to_string()));
	assert_eq!(values.get("HIGH_ONLY"), Some(&"value".to_string()));
}

#[test]
fn test_prefixed_source() {
	let mut inner = MurMemorySource::new();
	inner.set("KEY", "value");

	let source = MurPrefixedSource::new(inner, "PREFIX_");
	let values = source.load().unwrap();

	assert_eq!(values.get("PREFIX_KEY"), Some(&"value".to_string()));
	assert_eq!(values.get("KEY"), None);
}

#[test]
fn test_parse_json() {
	let content = r#"{"database": {"host": "localhost", "port": 5432}}"#;
	let values = MurFileSource::parse_json(content).unwrap();

	assert_eq!(values.get("DATABASE_HOST"), Some(&"localhost".to_string()));
	assert_eq!(values.get("DATABASE_PORT"), Some(&"5432".to_string()));
}
