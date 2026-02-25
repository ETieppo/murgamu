use super::pattern::MurRoutePattern;
use crate::server::aliases::MurRouteHandler;
use crate::server::guard::MurGuard;
use crate::server::interceptor::MurInterceptor;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct MurRouteAccessControl {
	pub is_public: bool,
	pub allowed_roles: HashSet<String>,
}

pub(crate) struct MurRouteEntry {
	pub pattern: MurRoutePattern,
	pub handler: MurRouteHandler,
	pub guards: Vec<Arc<dyn MurGuard + Send + Sync>>,
	pub interceptors: Vec<Arc<dyn MurInterceptor + Send + Sync>>,
	pub metadata: HashMap<String, String>,
	pub access_control: MurRouteAccessControl,
}

impl MurRouteEntry {
	pub fn new(pattern: MurRoutePattern, handler: MurRouteHandler) -> Self {
		Self {
			pattern,
			handler,
			guards: Vec::new(),
			interceptors: Vec::new(),
			metadata: HashMap::new(),
			access_control: MurRouteAccessControl::default(),
		}
	}
}
