// use std::time::Duration;

// use super::{
// 	// always_indicator::MurAlwaysHealthyIndicator,
// 	builder::MurHealthBuilder,
// 	check::MurHealthCheck,
// 	config::MurHealthConfig,
// 	indicator_result::MurHealthIndicatorResult,
// 	response::MurHealthResponse,
// 	status::MurHealthStatus,
// };

// #[test]
// fn test_health_status_combine() {
// 	assert_eq!(
// 		MurHealthStatus::Up.combine(MurHealthStatus::Up),
// 		MurHealthStatus::Up
// 	);
// 	assert_eq!(
// 		MurHealthStatus::Up.combine(MurHealthStatus::Down),
// 		MurHealthStatus::Down
// 	);
// 	assert_eq!(
// 		MurHealthStatus::Degraded.combine(MurHealthStatus::Up),
// 		MurHealthStatus::Degraded
// 	);
// 	assert_eq!(
// 		MurHealthStatus::Unknown.combine(MurHealthStatus::Degraded),
// 		MurHealthStatus::Unknown
// 	);
// }

// #[test]
// fn test_health_status_http_code() {
// 	assert_eq!(MurHealthStatus::Up.http_status_code(), 200);
// 	assert_eq!(MurHealthStatus::Degraded.http_status_code(), 200);
// 	assert_eq!(MurHealthStatus::Down.http_status_code(), 503);
// 	assert_eq!(MurHealthStatus::Unknown.http_status_code(), 503);
// }

// #[test]
// fn test_health_indicator_result() {
// 	let result = MurHealthIndicatorResult::healthy()
// 		.detail("key", "value")
// 		.duration(Duration::from_millis(100));

// 	assert_eq!(result.status, MurHealthStatus::Up);
// 	assert_eq!(result.duration_ms, Some(100));
// 	assert!(result.details.is_some());
// }

// #[test]
// fn test_health_builder() {
// 	let health = MurHealthBuilder::new()
// 		.path("/api/health")
// 		.version("1.0.0")
// 		.include_details(true)
// 		.include_timestamp(true)
// 		.build();

// 	assert_eq!(health.path(), "/api/health");
// }

// #[test]
// fn test_health_response() {
// 	let response = MurHealthResponse::healthy();
// 	assert_eq!(response.status, MurHealthStatus::Up);
// 	assert!(response.timestamp.is_some());

// 	let response = MurHealthResponse::unhealthy();
// 	assert_eq!(response.status, MurHealthStatus::Down);
// }

// #[test]
// fn test_health_config_default() {
// 	let config = MurHealthConfig::default();
// 	assert_eq!(config.path, "/health");
// 	assert!(config.include_details);
// 	assert!(config.parallel);
// }

// // TODO:
// // FIXME:
// // #[tokio::test]
// // async fn test_always_healthy_indicator() {
// // 	let indicator = MurAlwaysHealthyIndicator;
// // 	let result = indicator.check().await;
// // 	assert_eq!(result.status, MurHealthStatus::Up);
// // }

// #[tokio::test]
// async fn test_health_check_liveness() {
// 	let health = MurHealthCheck::new();
// 	let response = health.check_liveness().await;
// 	assert_eq!(response.status, MurHealthStatus::Up);
// }

// #[test]
// fn test_health_status_display() {
// 	assert_eq!(format!("{}", MurHealthStatus::Up), "up");
// 	assert_eq!(format!("{}", MurHealthStatus::Down), "down");
// 	assert_eq!(format!("{}", MurHealthStatus::Degraded), "degraded");
// 	assert_eq!(format!("{}", MurHealthStatus::Unknown), "unknown");
// }

// #[test]
// fn test_health_status_is_healthy() {
// 	assert!(MurHealthStatus::Up.is_healthy());
// 	assert!(MurHealthStatus::Degraded.is_healthy());
// 	assert!(!MurHealthStatus::Down.is_healthy());
// 	assert!(!MurHealthStatus::Unknown.is_healthy());
// }
