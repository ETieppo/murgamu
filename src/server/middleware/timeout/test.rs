use super::*;
use hyper::StatusCode;
use std::time::Duration;

#[test]
fn test_timeout_config_default() {
	let config = TimeoutConfig::default();
	assert_eq!(config.timeout, Duration::from_secs(30));
	assert_eq!(config.status_code, StatusCode::REQUEST_TIMEOUT);
	assert!(config.message.is_none());
	assert!(config.skip_paths.is_empty());
}

#[test]
fn test_timeout_config_builder() {
	let config = TimeoutConfig::from_secs(60)
		.status_code(StatusCode::GATEWAY_TIMEOUT)
		.message("Custom timeout")
		.skip_path("/health")
		.skip_path_prefix("/api/long")
		.include_timeout_header(true);

	assert_eq!(config.timeout, Duration::from_secs(60));
	assert_eq!(config.status_code, StatusCode::GATEWAY_TIMEOUT);
	assert_eq!(config.message, Some("Custom timeout".to_string()));
	assert!(config.skip_paths.contains(&"/health".to_string()));
	assert!(config.skip_path_prefixes.contains(&"/api/long".to_string()));
	assert!(config.include_timeout_header);
}

#[test]
fn test_timeout_middleware_builder() {
	let timeout = MurTimeout::from_secs(30)
		.gateway_timeout()
		.with_message("Gateway timed out")
		.skip_path("/health")
		.include_header();

	assert_eq!(timeout.config.timeout, Duration::from_secs(30));
	assert_eq!(timeout.config.status_code, StatusCode::GATEWAY_TIMEOUT);
	assert_eq!(
		timeout.config.message,
		Some("Gateway timed out".to_string())
	);
}

#[test]
fn test_should_skip() {
	let timeout = MurTimeout::from_secs(30)
		.skip_path("/health")
		.skip_path("/metrics")
		.skip_prefix("/api/uploads");

	assert!(timeout.should_skip("/health"));
	assert!(timeout.should_skip("/metrics"));
	assert!(timeout.should_skip("/api/uploads/file"));
	assert!(!timeout.should_skip("/api/users"));
}

#[test]
fn test_convenience_functions() {
	let short = MurTimeout::from_secs(5);
	assert_eq!(short.config.timeout, Duration::from_secs(5));

	let medium = MurTimeout::from_secs(30);
	assert_eq!(medium.config.timeout, Duration::from_secs(30));

	let long = MurTimeout::from_secs(120);
	assert_eq!(long.config.timeout, Duration::from_secs(120));

	let gateway = MurTimeout::from_secs(60).gateway_timeout();
	assert_eq!(gateway.config.status_code, StatusCode::GATEWAY_TIMEOUT);
}
