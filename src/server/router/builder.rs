use super::MurRouteInfo;
use super::core::MurRouter;
use super::entry::MurRouteEntry;
use super::pattern::MurRoutePattern;
use crate::server::aliases::MurRouteHandler;
use crate::server::guard::MurGuard;
use crate::server::interceptor::MurInterceptor;
use std::collections::HashMap;
use std::sync::Arc;

pub struct MurRouteBuilder<'a> {
	router: &'a mut MurRouter,
	method: String,
	path: String,
	guards: Vec<Arc<dyn MurGuard + Sync + Send>>,
	interceptors: Vec<Arc<dyn MurInterceptor + Sync + Send>>,
	metadata: HashMap<String, String>,
}

impl<'a> MurRouteBuilder<'a> {
	pub fn new(router: &'a mut MurRouter, method: &str, path: &str) -> Self {
		Self {
			router,
			method: method.to_uppercase(),
			path: path.to_string(),
			guards: Vec::new(),
			interceptors: Vec::new(),
			metadata: HashMap::new(),
		}
	}

	pub fn guard(mut self, guard: impl MurGuard + 'static) -> Self {
		self.guards.push(Arc::new(guard));
		self
	}

	pub fn interceptor(mut self, interceptor: impl MurInterceptor + 'static) -> Self {
		self.interceptors.push(Arc::new(interceptor));
		self
	}

	pub fn metadata(mut self, key: &str, value: &str) -> Self {
		self.metadata.insert(key.to_string(), value.to_string());
		self
	}

	pub fn handler(self, handler: MurRouteHandler) {
		let pattern = MurRoutePattern::new(&self.path);
		let mut entry = MurRouteEntry::new(pattern, handler);
		entry.guards = self.guards;
		entry.interceptors = self.interceptors;
		entry.metadata = self.metadata;

		self.router.route_info.push(MurRouteInfo {
			method: self.method.clone(),
			path: self.path,
			controller: String::from("manual"),
			handler: String::new(),
		});

		if !self.router.registered_methods.contains(&self.method) {
			self.router.registered_methods.push(self.method.clone());
		}

		self.router
			.routes_by_method
			.entry(self.method)
			.or_default()
			.push(entry);

		self.router.sort_all_routes();
	}
}
