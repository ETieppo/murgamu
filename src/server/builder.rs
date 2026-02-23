use crate::MurController;

use super::config::MurServerConfig;
use super::error::MurExceptionFilter;
use super::guard::MurGuard;
use super::interceptor::MurInterceptor;
use super::middleware::MurMiddleware;
use super::module::MurModule;
use super::router::MurRouter;
use super::runner::MurServerRunner;
use super::security::tls::MurTlsAcceptor;
use super::security::tls::MurTlsConfig;
use super::service::MurInjectable;
use super::service::MurInjects;
use super::service::MurService;
use super::service::MurServiceContainer;
use std::collections::HashSet;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;

pub struct MurServer {
	modules: Vec<Box<dyn MurModule>>,
	container: MurServiceContainer,
	injects: MurInjects,
	guards: Vec<Box<dyn MurGuard>>,
	interceptors: Vec<Box<dyn MurInterceptor>>,
	middleware: Vec<Box<dyn MurMiddleware>>,
	exception_filters: Vec<Box<dyn MurExceptionFilter>>,
	config: MurServerConfig,
	on_startup: Vec<Box<dyn Fn() + Send + Sync>>,
	on_shutdown: Vec<Box<dyn Fn() + Send + Sync>>,
}

impl MurServer {
	pub fn new() -> Self {
		Self {
			modules: Vec::new(),
			container: MurServiceContainer::new(),
			guards: Vec::new(),
			injects: MurInjects::new(),
			interceptors: Vec::new(),
			middleware: Vec::new(),
			exception_filters: Vec::new(),
			config: MurServerConfig::default(),
			on_startup: Vec::new(),
			on_shutdown: Vec::new(),
		}
	}

	pub fn with_config(config: MurServerConfig) -> Self {
		Self {
			modules: Vec::new(),
			container: MurServiceContainer::new(),
			injects: MurInjects::new(),
			guards: Vec::new(),
			interceptors: Vec::new(),
			middleware: Vec::new(),
			exception_filters: Vec::new(),
			config,
			on_startup: Vec::new(),
			on_shutdown: Vec::new(),
		}
	}

	fn build_imports_exports_container(
		module: &dyn MurModule,
		injects: &MurInjects,
	) -> MurServiceContainer {
		let mut out = MurServiceContainer::new();
		let mut visited = HashSet::<usize>::new();

		for imported in module.imports() {
			Self::fill_exports_rec_arc(imported, injects, &mut out, &mut visited);
		}

		out
	}

	fn fill_exports_rec_arc(
		module: Arc<dyn MurModule>,
		injects: &MurInjects,
		out: &mut MurServiceContainer,
		visited: &mut HashSet<usize>,
	) {
		let key = Arc::as_ptr(&module) as *const () as usize;
		if !visited.insert(key) {
			return;
		}

		for imported in module.imports() {
			Self::fill_exports_rec_arc(imported.clone(), injects, out, visited);
		}

		let services = module.services_with_injects(injects, out);
		let export_ids = module.exports();

		for (tid, svc) in services {
			if export_ids.contains(&tid) {
				out.register_dyn_with_id(tid, svc);
			}
		}
	}

	pub fn module(mut self, module: impl MurModule + 'static) -> Self {
		self.modules.push(Box::new(module));
		self
	}

	pub fn inject(mut self, inject: impl MurInjectable + 'static) -> Self {
		self.injects.register(inject);
		self
	}

	pub fn inject_arc<T: MurInjectable>(mut self, inject: Arc<T>) -> Self {
		self.injects.register_arc(inject);
		self
	}

	pub fn add_global_guard(mut self, guard: impl MurGuard + 'static) -> Self {
		self.guards.push(Box::new(guard));
		self
	}

	pub fn add_global_interceptor(mut self, interceptor: impl MurInterceptor + 'static) -> Self {
		self.interceptors.push(Box::new(interceptor));
		self
	}

	pub fn add_middleware(mut self, middleware: impl MurMiddleware + 'static) -> Self {
		self.middleware.push(Box::new(middleware));
		self
	}

	pub fn add_exception_filter(mut self, filter: impl MurExceptionFilter + 'static) -> Self {
		self.exception_filters.push(Box::new(filter));
		self
	}

	pub fn service<T: MurService>(mut self, service: T) -> Self {
		self.container.register(service);
		self
	}

	pub fn service_arc<T: MurService>(mut self, service: Arc<T>) -> Self {
		self.container.register_arc(service);
		self
	}

	pub fn configure(mut self, config: MurServerConfig) -> Self {
		self.config = config;
		self
	}

	pub fn addr(mut self, addr: impl Into<SocketAddr>) -> Self {
		self.config.addr = addr.into();
		self
	}

	pub fn enable_cors(mut self) -> Self {
		self.config = self.config.cors_all();
		self
	}

	pub fn cors(mut self, origins: Vec<String>) -> Self {
		self.config = self.config.cors(origins);
		self
	}

	pub fn no_logging(mut self) -> Self {
		self.config = self.config.no_logging();
		self
	}

	pub fn on_startup(mut self, hook: impl Fn() + Send + Sync + 'static) -> Self {
		self.on_startup.push(Box::new(hook));
		self
	}

	pub fn on_shutdown(mut self, hook: impl Fn() + Send + Sync + 'static) -> Self {
		self.on_shutdown.push(Box::new(hook));
		self
	}

	pub fn bind(self, addr: impl ToSocketAddrs) -> Result<MurServerRunner, std::io::Error> {
		let addr = addr.to_socket_addrs()?.next().ok_or_else(|| {
			std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid address")
		})?;

		self.bind_addr(addr)
	}

	pub fn bind_tls(
		mut self,
		addr: impl ToSocketAddrs,
		tls_config: MurTlsConfig,
	) -> Result<MurServerRunner, std::io::Error> {
		let addr = addr.to_socket_addrs()?.next().ok_or_else(|| {
			std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid address")
		})?;

		self.config.tls = Some(tls_config);
		self.bind_addr(addr)
	}

	pub fn bind_addr(mut self, addr: SocketAddr) -> Result<MurServerRunner, std::io::Error> {
		self.config.addr = addr;
		self.injects.on_init();

		let mut app_global = MurServiceContainer::new();
		app_global.merge(self.container);

		let mut runtime_global = app_global.clone();
		let mut controllers_to_register: Vec<std::sync::Arc<dyn MurController>> = Vec::new();

		for module in &self.modules {
			module.on_init();

			let mut visible = Self::build_imports_exports_container(module.as_ref(), &self.injects);
			visible.merge(app_global.clone());

			let local_services = module.services_with_injects(&self.injects, &visible);

			for (tid, svc) in local_services.iter() {
				visible.register_dyn_with_id(*tid, svc.clone());
			}

			for c in module.controllers_with_injects(&self.injects, &visible) {
				controllers_to_register.push(c);
			}

			runtime_global.merge(visible);
		}

		let container = Arc::new(runtime_global);
		let mut router = MurRouter::new(Arc::clone(&container));

		for c in controllers_to_register {
			router.register_controller(c);
		}

		for guard in self.guards {
			router.add_guard_boxed(guard);
		}

		for interceptor in self.interceptors {
			router.add_interceptor_boxed(interceptor);
		}

		for filter in self.exception_filters {
			router.exception_filters.push(Arc::from(filter));
		}

		for module in &self.modules {
			if self.config.enable_logging {
				println!("Loading module: {}", module.name());
			}

			for controller in module.controllers_with_injects(&self.injects, container.as_ref()) {
				router.register_controller(controller);
			}
		}

		if self.config.enable_logging {
			router.print_routes();
		}

		let tls_acceptor = if let Some(ref tls_config) = self.config.tls {
			Some(MurTlsAcceptor::new(tls_config)?)
		} else {
			None
		};

		Ok(MurServerRunner {
			router: Arc::new(router),
			config: self.config,
			modules: self.modules,
			injects: self.injects,
			on_startup: self.on_startup,
			on_shutdown: self.on_shutdown,
			tls_acceptor,
		})
	}
}

impl Default for MurServer {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_server_builder() {
		let server = MurServer::new().no_logging().enable_cors();

		assert!(!server.config.enable_logging);
		assert!(server.config.enable_cors);
	}
}
