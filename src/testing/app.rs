// TODO: analyse and fix
use super::client::MurTestClient;
use crate::container::core::MurServiceContainer;
use crate::container::injects::MurInjects;
use crate::mur_http::request::MurRequestContext;
use crate::router::MurRouter;
use crate::traits::{
	MurController, MurGuard, MurInjectable, MurInterceptor, MurMiddleware, MurModule, MurService,
};
use crate::types::MurRouteHandler;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MurTestConfig {
	pub verbose: bool,
	pub request_timeout_ms: u64,
	pub base_path: Option<String>,
	pub auto_json: bool,
	pub include_stack_traces: bool,
	pub env_vars: HashMap<String, String>,
}

impl Default for MurTestConfig {
	fn default() -> Self {
		Self {
			verbose: false,
			request_timeout_ms: 5000,
			base_path: None,
			auto_json: true,
			include_stack_traces: true,
			env_vars: HashMap::new(),
		}
	}
}

impl MurTestConfig {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn verbose(mut self) -> Self {
		self.verbose = true;
		self
	}

	pub fn timeout(mut self, timeout_ms: u64) -> Self {
		self.request_timeout_ms = timeout_ms;
		self
	}

	pub fn base_path(mut self, path: impl Into<String>) -> Self {
		self.base_path = Some(path.into());
		self
	}

	pub fn no_auto_json(mut self) -> Self {
		self.auto_json = false;
		self
	}

	pub fn no_stack_traces(mut self) -> Self {
		self.include_stack_traces = false;
		self
	}

	pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
		self.env_vars.insert(key.into(), value.into());
		self
	}

	pub fn envs(mut self, vars: impl IntoIterator<Item = (String, String)>) -> Self {
		self.env_vars.extend(vars);
		self
	}
}

pub struct MurTestApp {
	client: MurTestClient,
	container: Arc<MurServiceContainer>,
	router: Arc<MurRouter>,
	config: MurTestConfig,
}

impl MurTestApp {
	pub fn builder() -> MurTestAppBuilder {
		MurTestAppBuilder::new()
	}

	pub fn new(
		router: Arc<MurRouter>,
		container: Arc<MurServiceContainer>,
		config: MurTestConfig,
	) -> Self {
		let client = MurTestClient::new(Arc::clone(&router), Arc::clone(&container));

		Self {
			client,
			container,
			router,
			config,
		}
	}

	pub fn client(&self) -> &MurTestClient {
		&self.client
	}

	pub fn container(&self) -> &Arc<MurServiceContainer> {
		&self.container
	}

	pub fn service<T: MurService>(&self) -> Option<Arc<T>> {
		self.container.get::<T>()
	}

	pub fn router(&self) -> &Arc<MurRouter> {
		&self.router
	}

	pub fn config(&self) -> &MurTestConfig {
		&self.config
	}

	pub fn get(&self, path: &str) -> super::client::MurTestRequestBuilder {
		self.client.get(&self.resolve_path(path))
	}

	pub fn post(&self, path: &str) -> super::client::MurTestRequestBuilder {
		self.client.post(&self.resolve_path(path))
	}

	pub fn put(&self, path: &str) -> super::client::MurTestRequestBuilder {
		self.client.put(&self.resolve_path(path))
	}

	pub fn patch(&self, path: &str) -> super::client::MurTestRequestBuilder {
		self.client.patch(&self.resolve_path(path))
	}

	pub fn delete(&self, path: &str) -> super::client::MurTestRequestBuilder {
		self.client.delete(&self.resolve_path(path))
	}

	pub fn head(&self, path: &str) -> super::client::MurTestRequestBuilder {
		self.client.head(&self.resolve_path(path))
	}

	pub fn options(&self, path: &str) -> super::client::MurTestRequestBuilder {
		self.client.options(&self.resolve_path(path))
	}

	pub fn request(
		&self,
		method: http::Method,
		path: &str,
	) -> super::client::MurTestRequestBuilder {
		self.client.request(method, &self.resolve_path(path))
	}

	fn resolve_path(&self, path: &str) -> String {
		match &self.config.base_path {
			Some(base) => {
				let base = base.trim_end_matches('/');
				let path = path.trim_start_matches('/');
				format!("{}/{}", base, path)
			}
			None => path.to_string(),
		}
	}
}

pub struct MurTestAppBuilder {
	config: MurTestConfig,
	modules: Vec<Arc<dyn MurModule>>,
	controllers: Vec<Arc<dyn MurController>>,
	services: Vec<(TypeId, Arc<dyn MurService>)>,
	injects: MurInjects,
	guards: Vec<Arc<dyn MurGuard>>,
	interceptors: Vec<Arc<dyn MurInterceptor>>,
	middleware: Vec<Arc<dyn MurMiddleware>>,
	routes: Vec<(String, String, MurRouteHandler)>,
}

impl MurTestAppBuilder {
	pub fn new() -> Self {
		Self {
			config: MurTestConfig::default(),
			modules: Vec::new(),
			controllers: Vec::new(),
			services: Vec::new(),
			injects: MurInjects::new(),
			guards: Vec::new(),
			interceptors: Vec::new(),
			middleware: Vec::new(),
			routes: Vec::new(),
		}
	}

	pub fn config(mut self, config: MurTestConfig) -> Self {
		self.config = config;
		self
	}

	pub fn configure<F>(mut self, f: F) -> Self
	where
		F: FnOnce(MurTestConfig) -> MurTestConfig,
	{
		self.config = f(self.config);
		self
	}

	pub fn module<M: MurModule>(mut self, module: M) -> Self {
		self.modules.push(Arc::new(module));
		self
	}

	pub fn modules<I, M>(mut self, modules: I) -> Self
	where
		I: IntoIterator<Item = M>,
		M: MurModule,
	{
		for module in modules {
			self.modules.push(Arc::new(module));
		}
		self
	}

	pub fn controller<C: MurController>(mut self, controller: C) -> Self {
		self.controllers.push(Arc::new(controller));
		self
	}

	pub fn service<S: MurService>(mut self, service: S) -> Self {
		self.services
			.push((TypeId::of::<S>(), Arc::new(service) as Arc<dyn MurService>));
		self
	}

	pub fn service_for<T: 'static, S: MurService>(mut self, service: S) -> Self {
		self.services
			.push((TypeId::of::<T>(), Arc::new(service) as Arc<dyn MurService>));
		self
	}

	pub fn service_arc<S: MurService>(mut self, service: Arc<S>) -> Self {
		self.services
			.push((TypeId::of::<S>(), service as Arc<dyn MurService>));
		self
	}

	pub fn inject<I: MurInjectable>(mut self, inject: I) -> Self {
		self.injects.register(inject);
		self
	}

	pub fn inject_arc<I: MurInjectable>(mut self, inject: Arc<I>) -> Self {
		self.injects.register_arc(inject);
		self
	}

	pub fn guard<G: MurGuard>(mut self, guard: G) -> Self {
		self.guards.push(Arc::new(guard));
		self
	}

	pub fn interceptor<I: MurInterceptor>(mut self, interceptor: I) -> Self {
		self.interceptors.push(Arc::new(interceptor));
		self
	}

	pub fn middleware<M: MurMiddleware>(mut self, middleware: M) -> Self {
		self.middleware.push(Arc::new(middleware));
		self
	}

	pub fn route(
		mut self,
		method: impl Into<String>,
		path: impl Into<String>,
		handler: MurRouteHandler,
	) -> Self {
		self.routes.push((method.into(), path.into(), handler));
		self
	}

	pub fn build(self) -> MurTestApp {
		let mut container = MurServiceContainer::new();

		for module in &self.modules {
			for (_type_id, service) in module.services_with_injects(&self.injects) {
				container.register_dyn_service(service);
			}
		}

		for (_type_id, service) in self.services {
			container.register_dyn_service(service);
		}

		let container = Arc::new(container);

		let mut router = MurRouter::new(Arc::clone(&container));

		for module in &self.modules {
			for controller in module.controllers_with_injects(&self.injects) {
				for r in controller.routes(Arc::clone(&container)) {
					router.route(&r.method, &r.path, r.handler);
				}
			}
		}

		for controller in &self.controllers {
			for r in controller.clone().routes(Arc::clone(&container)) {
				router.route(&r.method, &r.path, r.handler);
			}
		}

		for (method, path, handler) in self.routes {
			router.route(&method, &path, handler);
		}

		let router = Arc::new(router);

		let mut client = MurTestClient::new(Arc::clone(&router), Arc::clone(&container));
		for mw in self.middleware {
			client = client.with_middleware(MiddlewareWrapper(mw));
		}

		MurTestApp {
			client,
			container,
			router,
			config: self.config,
		}
	}
}

impl Default for MurTestAppBuilder {
	fn default() -> Self {
		Self::new()
	}
}

struct MiddlewareWrapper(Arc<dyn MurMiddleware>);

impl MurMiddleware for MiddlewareWrapper {
	fn handle(
		&self,
		ctx: MurRequestContext,
		next: crate::traits::MurNext,
	) -> crate::types::MurFuture {
		self.0.handle(ctx, next)
	}

	fn name(&self) -> &str {
		self.0.name()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_config_defaults() {
		let config = MurTestConfig::default();
		assert!(!config.verbose);
		assert_eq!(config.request_timeout_ms, 5000);
		assert!(config.base_path.is_none());
		assert!(config.auto_json);
		assert!(config.include_stack_traces);
		assert!(config.env_vars.is_empty());
	}

	#[test]
	fn test_config_builder() {
		let config = MurTestConfig::new()
			.verbose()
			.timeout(10000)
			.base_path("/api/v1")
			.no_auto_json()
			.no_stack_traces()
			.env("TEST_VAR", "test_value");

		assert!(config.verbose);
		assert_eq!(config.request_timeout_ms, 10000);
		assert_eq!(config.base_path, Some("/api/v1".to_string()));
		assert!(!config.auto_json);
		assert!(!config.include_stack_traces);
		assert_eq!(
			config.env_vars.get("TEST_VAR"),
			Some(&"test_value".to_string())
		);
	}

	#[test]
	fn test_app_builder_new() {
		let builder = MurTestAppBuilder::new();
		assert!(builder.modules.is_empty());
		assert!(builder.controllers.is_empty());
		assert!(builder.services.is_empty());
	}

	#[test]
	fn test_resolve_path_without_base() {
		let container = Arc::new(MurServiceContainer::new());
		let app = MurTestApp::new(
			Arc::new(MurRouter::new(Arc::clone(&container))),
			container,
			MurTestConfig::new(),
		);

		assert_eq!(app.resolve_path("/users"), "/users");
		assert_eq!(app.resolve_path("users"), "users");
	}

	#[test]
	fn test_resolve_path_with_base() {
		let config = MurTestConfig::new().base_path("/api/v1");
		let container = Arc::new(MurServiceContainer::new());
		let app = MurTestApp::new(
			Arc::new(MurRouter::new(Arc::clone(&container))),
			container,
			config,
		);

		assert_eq!(app.resolve_path("/users"), "/api/v1/users");
		assert_eq!(app.resolve_path("users"), "/api/v1/users");
	}

	#[test]
	fn test_resolve_path_with_trailing_slash_base() {
		let config = MurTestConfig::new().base_path("/api/v1/");
		let container = Arc::new(MurServiceContainer::new());
		let app = MurTestApp::new(
			Arc::new(MurRouter::new(Arc::clone(&container))),
			container,
			config,
		);

		assert_eq!(app.resolve_path("/users"), "/api/v1/users");
	}
}
