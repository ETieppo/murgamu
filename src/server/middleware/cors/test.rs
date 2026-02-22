use std::collections::HashSet;
use super::AllowedOrigins;

#[test]
fn test_allowed_origins_any() {
	let origins = AllowedOrigins::Any;
	assert!(origins.is_allowed("https://example.com"));
	assert!(origins.is_allowed("http://localhost:3000"));
	assert_eq!(
		origins.header_value(Some("https://example.com")),
		Some("*".to_string())
	);
}

#[test]
fn test_allowed_origins_list() {
	let mut set = HashSet::new();
	set.insert("https://example.com".to_string());
	let origins = AllowedOrigins::List(set);

	assert!(origins.is_allowed("https://example.com"));
	assert!(!origins.is_allowed("https://other.com"));
	assert_eq!(
		origins.header_value(Some("https://example.com")),
		Some("https://example.com".to_string())
	);
	assert_eq!(origins.header_value(Some("https://other.com")), None);
}

#[test]
fn test_allowed_origins_mirror() {
	let origins = AllowedOrigins::Mirror;
	assert!(origins.is_allowed("https://example.com"));
	assert_eq!(
		origins.header_value(Some("https://example.com")),
		Some("https://example.com".to_string())
	);
}

// TODO:
// FIXME:
// #[test]
// fn test_cors_builder() {
// 	let cors = MurCors::new()
// 		.allow_origin("https://example.com")
// 		.allow_methods(vec!["GET", "POST"])
// 		.allow_credentials(true)
// 		.max_age(3600);

// 	assert!(cors.is_origin_allowed("https://example.com"));
// 	assert!(!cors.is_origin_allowed("https://other.com"));
// 	assert!(cors.is_method_allowed(&Method::GET));
// 	assert!(cors.is_method_allowed(&Method::POST));
// 	assert!(!cors.is_method_allowed(&Method::DELETE));
// }

// #[test]
// fn test_cors_permissive() {
// 	let cors = MurCors::permissive();
// 	assert!(cors.is_origin_allowed("https://anything.com"));
// 	assert!(cors.is_method_allowed(&Method::GET));
// 	assert!(cors.is_method_allowed(&Method::POST));
// 	assert!(cors.is_method_allowed(&Method::DELETE));
// }

// #[test]
// fn test_credentials_changes_origin_mode() {
// 	let cors = MurCors::new().allow_any_origin().allow_credentials(true);

// 	assert!(matches!(
// 		cors.config.allowed_origins,
// 		AllowedOrigins::Mirror
// 	));
// }
