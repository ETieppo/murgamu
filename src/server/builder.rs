use crate::MurGuardFactory;
use crate::MurPipeDyn;
use crate::server::interceptor::MurInterceptorFactory;
use crate::server::pipe::MurPipeFactory;

use super::config::MurServerConfig;
use super::guard::MurGuard;
use super::interceptor::MurInterceptor;
use super::middleware::MurMiddleware;
use super::module::MurModule;
use super::router::MurRouter;
use super::runner::MurServerRunner;
use super::security::tls::{MurTlsAcceptor, MurTlsConfig};
use super::service::{MurInjectable, MurInjects, MurService, MurServiceContainer};
use std::collections::HashSet;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;

type GuardFactory =
	Box<dyn Fn(&MurInjects, &MurServiceContainer) -> Box<dyn MurGuard + Send + Sync> + Send + Sync>;

type PipeFactory = Box<
	dyn Fn(&MurInjects, &MurServiceContainer) -> Box<dyn MurPipeDyn + Send + Sync> + Send + Sync,
>;

type InterceptorFactory = Box<
	dyn Fn(&MurInjects, &MurServiceContainer) -> Box<dyn MurInterceptor + Send + Sync>
		+ Send
		+ Sync,
>;

pub struct MurServer {
	modules: Vec<Box<dyn MurModule + Send + Sync>>,
	container: MurServiceContainer,
	injects: MurInjects,
	guards: Vec<GuardFactory>,
	pipes: Vec<PipeFactory>,
	/// Interceptors created via DI factory (`.interceptor::<T>()`).
	interceptor_factories: Vec<InterceptorFactory>,
	/// Interceptors provided as ready-made instances (`.add_global_interceptor(instance)`).
	interceptor_instances: Vec<Box<dyn MurInterceptor + Send + Sync>>,
	middleware: Vec<Box<dyn MurMiddleware + Sync + Send>>,
	config: MurServerConfig,
	on_startup: Vec<Box<dyn Fn() + Send + Sync>>,
	on_shutdown: Vec<Box<dyn Fn() + Send + Sync>>,
}

impl Default for MurServer {
	fn default() -> Self {
		Self::new()
	}
}

impl MurServer {
	pub fn new() -> Self {
		Self {
			modules: Vec::new(),
			container: MurServiceContainer::new(),
			injects: MurInjects::new(),
			guards: Vec::new(),
			pipes: Vec::new(),
			interceptor_factories: Vec::new(),
			interceptor_instances: Vec::new(),
			middleware: Vec::new(),
			config: MurServerConfig::default(),
			on_startup: Vec::new(),
			on_shutdown: Vec::new(),
		}
	}

	pub fn configure(mut self, config: MurServerConfig) -> Self {
		self.config = config;
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

	pub fn service<T: MurService + Send + Sync>(mut self, service: T) -> Self {
		self.container.register(service);
		self
	}

	pub fn service_arc<T: MurService + Send + Sync>(mut self, service: Arc<T>) -> Self {
		self.container.register_arc(service);
		self
	}

	pub fn guard<T>(mut self) -> Self
	where
		T: MurGuardFactory + Send + Sync + 'static,
	{
		self.guards.push(Box::new(|injects, container| {
			Box::new(T::create(injects, container))
		}));
		self
	}

	pub fn pipe<T>(mut self) -> Self
	where
		T: MurPipeFactory + MurPipeDyn + 'static,
	{
		self.pipes.push(Box::new(|injects, container| {
			Box::new(T::create(injects, container)) as Box<dyn MurPipeDyn>
		}));
		self
	}

	/// Register an interceptor that participates in DI.
	/// Use this for interceptors declared with `#[interceptor]` that have injected deps.
	pub fn interceptor<T>(mut self) -> Self
	where
		T: MurInterceptorFactory + Send + Sync + 'static,
	{
		self.interceptor_factories.push(Box::new(|injects, container| {
			Box::new(T::create(injects, container)) as Box<dyn MurInterceptor + Send + Sync>
		}));
		self
	}

	/// Register a manually constructed interceptor (no DI).
	pub fn add_global_interceptor(mut self, interceptor: impl MurInterceptor + 'static) -> Self {
		self.interceptor_instances.push(Box::new(interceptor));
		self
	}

	pub fn add_middleware(mut self, middleware: impl MurMiddleware + 'static) -> Self {
		self.middleware.push(Box::new(middleware));
		self
	}

	pub fn bind(self, addr: impl ToSocketAddrs) -> Result<MurServerRunner, std::io::Error> {
		let addr = resolve_addr(addr)?;
		self.bind_addr(addr)
	}

	pub fn bind_tls(
		mut self,
		addr: impl ToSocketAddrs,
		tls_config: MurTlsConfig,
	) -> Result<MurServerRunner, std::io::Error> {
		let addr = resolve_addr(addr)?;
		self.config.tls = Some(tls_config);
		self.bind_addr(addr)
	}

	pub fn bind_addr(mut self, addr: SocketAddr) -> Result<MurServerRunner, std::io::Error> {
		self.config.addr = addr;
		self.injects.on_init();

		let mut global = MurServiceContainer::new();
		global.merge(self.container);

		let mut runtime = global.clone();
		for module in &self.modules {
			module.on_init();

			let mut visible =
				Self::resolve_module_container(module.as_ref(), &self.injects, &global);
			visible.merge(global.clone());

			let local_services = module.services_with_injects(&self.injects, &visible);
			for (tid, svc) in &local_services {
				visible.register_dyn_with_id(*tid, svc.clone());
			}

			runtime.merge(visible);
		}

		let container = Arc::new(runtime);
		let mut router = MurRouter::new(Arc::clone(&container));

		for factory in self.guards {
			router.add_guard_boxed(factory(&self.injects, &container));
		}
		for factory in self.pipes {
			router.add_pipe_boxed(factory(&self.injects, &container));
		}
		for factory in self.interceptor_factories {
			router.add_interceptor_boxed(factory(&self.injects, &container));
		}
		for instance in self.interceptor_instances {
			router.add_interceptor_boxed(instance);
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

		let tls_acceptor = self
			.config
			.tls
			.as_ref()
			.map(MurTlsAcceptor::new)
			.transpose()?;

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

	fn resolve_module_container(
		module: &dyn MurModule,
		injects: &MurInjects,
		base: &MurServiceContainer,
	) -> MurServiceContainer {
		let mut out = base.clone();
		let mut visited = HashSet::<usize>::new();

		for imported in module.imports() {
			Self::collect_exports(imported, injects, &mut out, &mut visited);
		}

		out
	}

	fn collect_exports(
		module: Arc<dyn MurModule + Send + Sync>,
		injects: &MurInjects,
		out: &mut MurServiceContainer,
		visited: &mut HashSet<usize>,
	) {
		let key = Arc::as_ptr(&module) as *const () as usize;
		if !visited.insert(key) {
			return;
		}

		for imported in module.imports() {
			Self::collect_exports(imported, injects, out, visited);
		}

		let services = module.services_with_injects(injects, out);
		let export_ids = module.exports();

		for (tid, svc) in services {
			if export_ids.contains(&tid) {
				out.register_dyn_with_id(tid, svc);
			}
		}
	}
}

fn resolve_addr(addr: impl ToSocketAddrs) -> Result<SocketAddr, std::io::Error> {
	addr.to_socket_addrs()?
		.next()
		.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid address"))
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

	#[test]
	fn test_server_default() {
		let server = MurServer::default();
		assert!(server.modules.is_empty());
		assert!(server.guards.is_empty());
	}
}
