use super::*;

#[test]
fn test_profile_from_str() {
	assert_eq!(
		MurEnvProfile::from_str("development"),
		MurEnvProfile::Development
	);
	assert_eq!(MurEnvProfile::from_str("dev"), MurEnvProfile::Development);
	assert_eq!(MurEnvProfile::from_str("local"), MurEnvProfile::Development);
	assert_eq!(MurEnvProfile::from_str("staging"), MurEnvProfile::Staging);
	assert_eq!(MurEnvProfile::from_str("stg"), MurEnvProfile::Staging);
	assert_eq!(
		MurEnvProfile::from_str("production"),
		MurEnvProfile::Production
	);
	assert_eq!(MurEnvProfile::from_str("prod"), MurEnvProfile::Production);
	assert_eq!(MurEnvProfile::from_str("test"), MurEnvProfile::Test);
	assert_eq!(MurEnvProfile::from_str("testing"), MurEnvProfile::Test);
}

#[test]
fn test_profile_as_str() {
	assert_eq!(MurEnvProfile::Development.as_str(), "development");
	assert_eq!(MurEnvProfile::Staging.as_str(), "staging");
	assert_eq!(MurEnvProfile::Production.as_str(), "production");
	assert_eq!(MurEnvProfile::Test.as_str(), "test");
}

#[test]
fn test_profile_checks() {
	assert!(MurEnvProfile::Development.is_development());
	assert!(MurEnvProfile::Production.is_production());
	assert!(MurEnvProfile::Staging.is_staging());
	assert!(MurEnvProfile::Test.is_test());

	assert!(MurEnvProfile::Production.is_production_like());
	assert!(MurEnvProfile::Staging.is_production_like());
	assert!(!MurEnvProfile::Development.is_production_like());

	assert!(MurEnvProfile::Development.is_development_like());
	assert!(MurEnvProfile::Test.is_development_like());
	assert!(!MurEnvProfile::Production.is_development_like());
}

#[test]
fn test_env_file() {
	assert_eq!(MurEnvProfile::Development.env_file(), ".env.development");
	assert_eq!(MurEnvProfile::Production.env_file(), ".env.production");
}

#[test]
fn test_env_files() {
	let files = MurEnvProfile::Development.env_files();
	assert_eq!(files.len(), 3);
	assert_eq!(files[0], ".env");
	assert_eq!(files[1], ".env.development");
	assert_eq!(files[2], ".env.local");
}

#[test]
fn test_server_defaults() {
	let dev = MurEnvProfile::Development.server_defaults();
	assert_eq!(dev.host, "127.0.0.1");
	assert_eq!(dev.port, 3000);
	assert!(dev.enable_cors);

	let prod = MurEnvProfile::Production.server_defaults();
	assert_eq!(prod.host, "0.0.0.0");
	assert_eq!(prod.port, 8080);
	assert!(!prod.enable_cors);
}

#[test]
fn test_custom_profile() {
	let custom = MurEnvProfile::from_str("qa");
	assert!(custom.is_custom());
	assert_eq!(custom.as_str(), "qa");
	assert_eq!(custom.env_file(), ".env.qa");
}

#[test]
fn test_display() {
	assert_eq!(format!("{}", MurEnvProfile::Development), "development");
	assert_eq!(format!("{}", MurEnvProfile::Production), "production");
}
