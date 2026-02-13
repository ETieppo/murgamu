use super::pattern::MurRoutePattern;
use crate::traits::{MurGuard, MurInterceptor};
use crate::types::MurRouteHandler;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) struct MurRouteEntry {
	pub pattern: MurRoutePattern,
	pub handler: MurRouteHandler,
	pub guards: Vec<Arc<dyn MurGuard>>,
	pub interceptors: Vec<Arc<dyn MurInterceptor>>,
	#[allow(dead_code)]
	pub metadata: HashMap<String, String>,
}

impl MurRouteEntry {
	pub fn new(pattern: MurRoutePattern, handler: MurRouteHandler) -> Self {
		Self {
			pattern,
			handler,
			guards: Vec::new(),
			interceptors: Vec::new(),
			metadata: HashMap::new(),
		}
	}
}
