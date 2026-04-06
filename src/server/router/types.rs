use crate::server::aliases::MurRouteHandler;

#[derive(Debug, Clone)]
pub struct MurRouteInfo {
	pub method: String,
	pub path: String,
	pub controller: String,
	pub handler: String,
}

#[derive(Clone)]
pub struct MurRouteDefinition {
	pub method: String,
	pub path: String,
	pub handler: MurRouteHandler,
	pub is_public: bool,
	pub allowed_roles: Vec<String>,
}
