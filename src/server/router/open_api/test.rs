use super::{
	mur_open_api::MurOpenApi, operation::MurApiOperation, schema::MurApiSchema,
};

#[test]
fn test_schema_string() {
	let schema = MurApiSchema::string();
	assert_eq!(schema.schema_type, Some("string".to_string()));
}

#[test]
fn test_schema_object() {
	let schema = MurApiSchema::object()
		.required_property("id", MurApiSchema::string())
		.property("name", MurApiSchema::string().nullable());

	assert_eq!(schema.schema_type, Some("object".to_string()));
	assert!(schema.properties.contains_key("id"));
	assert!(schema.properties.contains_key("name"));
	assert!(schema.required.contains(&"id".to_string()));
}

#[test]
fn test_openapi_builder() {
	let spec = MurOpenApi::new("Test API", "1.0.0")
		.description("A test API")
		.server("https://api.test.com", "Production")
		.tag("users", "User operations")
		.bearer_auth()
		.path("/users", |p| {
			p.get(MurApiOperation::new("List users").tag("users"))
				.post(MurApiOperation::new("Create user").tag("users"))
		})
		.build();

	assert_eq!(spec.info.title, "Test API");
	assert_eq!(spec.info.version, "1.0.0");
	assert_eq!(spec.servers.len(), 1);
	assert_eq!(spec.tags.len(), 1);
	assert!(spec.paths.contains_key("/users"));
}

#[test]
fn test_operation_builder() {
	let op = MurApiOperation::new("Get user")
		.operation_id("getUser")
		.description("Get a user by ID")
		.tag("users")
		.path_param("id", "User ID")
		.response(200, "User found")
		.response(404, "User not found")
		.bearer_auth();

	assert_eq!(op.summary, Some("Get user".to_string()));
	assert_eq!(op.operation_id, Some("getUser".to_string()));
	assert!(op.tags.contains(&"users".to_string()));
	assert_eq!(op.parameters.len(), 1);
	assert!(op.responses.contains_key("200"));
	assert!(op.responses.contains_key("404"));
}

#[test]
fn test_to_json() {
	let spec = MurOpenApi::new("Test", "1.0.0").build();
	let json = spec.to_json();
	assert!(json.is_ok());
	assert!(json.unwrap().contains("\"openapi\": \"3.0.3\""));
}
