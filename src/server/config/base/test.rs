use super::*;

#[test]
fn test_parse_dotenv_simple() {
	let mut config = MurConfig::new();
	config.parse_dotenv("KEY=value\nANOTHER=test");
	assert_eq!(config.get("KEY"), Some(&"value".to_string()));
	assert_eq!(config.get("ANOTHER"), Some(&"test".to_string()));
}

#[test]
fn test_parse_dotenv_quoted() {
	let mut config = MurConfig::new();
	config.parse_dotenv(r#"KEY="hello world""#);
	assert_eq!(config.get("KEY"), Some(&"hello world".to_string()));
}

#[test]
fn test_parse_dotenv_comments() {
	let mut config = MurConfig::new();
	config.parse_dotenv("# This is a comment\nKEY=value # inline comment");
	assert_eq!(config.get("KEY"), Some(&"value".to_string()));
}

#[test]
fn test_get_or() {
	let mut config = MurConfig::new();
	config.set("PORT", "8080");
	assert_eq!(config.get_or::<u16>("PORT", 3000), 8080);
	assert_eq!(config.get_or::<u16>("MISSING", 3000), 3000);
}

#[test]
fn test_get_bool() {
	let mut config = MurConfig::new();
	config.set("DEBUG", "true");
	config.set("VERBOSE", "1");
	config.set("DISABLED", "false");

	assert!(config.get_bool_or("DEBUG", false));
	assert!(config.get_bool_or("VERBOSE", false));
	assert!(!config.get_bool_or("DISABLED", true));
	assert!(!config.get_bool_or("MISSING", false));
}

#[test]
fn test_get_list() {
	let mut config = MurConfig::new();
	config.set("HOSTS", "host1,host2,host3");
	let hosts = config.get_list("HOSTS").unwrap();
	assert_eq!(hosts, vec!["host1", "host2", "host3"]);
}

#[test]
fn test_parse_duration() {
	assert_eq!(
		parse_duration("30"),
		Some(std::time::Duration::from_secs(30))
	);
	assert_eq!(
		parse_duration("30s"),
		Some(std::time::Duration::from_secs(30))
	);
	assert_eq!(
		parse_duration("5m"),
		Some(std::time::Duration::from_secs(300))
	);
	assert_eq!(
		parse_duration("2h"),
		Some(std::time::Duration::from_secs(7200))
	);
}

#[test]
fn test_parse_size() {
	assert_eq!(parse_size("1024"), Some(1024));
	assert_eq!(parse_size("1KB"), Some(1024));
	assert_eq!(parse_size("1MB"), Some(1024 * 1024));
	assert_eq!(parse_size("1GB"), Some(1024 * 1024 * 1024));
}

#[test]
fn test_validate_required() {
	let mut config = MurConfig::new();
	config.set("KEY1", "value1");
	config.set("KEY2", "value2");

	assert!(config.validate_required(&["KEY1", "KEY2"]).is_ok());
	assert!(config.validate_required(&["KEY1", "MISSING"]).is_err());
}

#[test]
fn test_subset() {
	let mut config = MurConfig::new();
	config.set("DATABASE_HOST", "localhost");
	config.set("DATABASE_PORT", "5432");
	config.set("OTHER_KEY", "value");

	let db_config = config.subset("DATABASE_");
	assert_eq!(db_config.get("HOST"), Some(&"localhost".to_string()));
	assert_eq!(db_config.get("PORT"), Some(&"5432".to_string()));
	assert_eq!(db_config.get("OTHER_KEY"), None);
}

#[test]
fn test_builder() {
	let config = MurConfigBuilder::new()
		.no_env()
		.default("PORT", "3000")
		.set("HOST", "localhost")
		.build()
		.unwrap();

	assert_eq!(config.get_or::<u16>("PORT", 0), 3000);
	assert_eq!(config.get_or_string("HOST", ""), "localhost");
}
