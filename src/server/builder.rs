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

/// The main entry point for building and starting a Murgamu HTTP server.
///
/// `MurServer` uses a fluent builder pattern to compose modules, services,
/// guards, interceptors, middleware, and configuration before binding to a
/// network address and running the async event loop.
///
/// # Minimal example
///
/// ```rust,ignore
/// #[murgamu::main]
/// async fn main() -> MurMainResult {
///     MurServer::new()
///         .module(AppModule::new())
///         .bind(("0.0.0.0", 3000))?
///         .run()
///         .await
/// }
/// ```
///
/// # `.env` loading
///
/// [`new`](Self::new) automatically loads `.env`, `.env.{profile}`, and
/// `.env.local` into the process environment so that [`mur_env`](crate::mur_env)
/// works for the rest of the application startup.
pub struct MurServer {
	modules: Vec<Box<dyn MurModule + Send + Sync>>,
	container: MurServiceContainer,
	injects: MurInjects,
	guards: Vec<GuardFactory>,
	pipes: Vec<PipeFactory>,
	interceptor_factories: Vec<InterceptorFactory>,
	interceptor_instances: Vec<Box<dyn MurInterceptor + Send + Sync>>,
	middleware: Vec<Box<dyn MurMiddleware + Sync + Send>>,
	config: MurServerConfig,
	on_startup: Vec<Box<dyn Fn() + Send + Sync>>,
	on_shutdown: Vec<Box<dyn Fn() + Send + Sync>>,
	default_public: bool,
}

impl Default for MurServer {
	fn default() -> Self {
		Self::new()
	}
}

impl MurServer {
	/// Creates a new `MurServer` builder with default settings.
	///
	/// As a side-effect, loads `.env` files into the process environment so
	/// that [`mur_env`](crate::mur_env) is available immediately after this call.
	pub fn new() -> Self {
		// SAFETY: called at program startup before any additional threads are spawned.
		unsafe { crate::server::config::MurEnv::load() };

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
			default_public: false,
		}
	}

	/// Makes all routes public by default (no authentication required).
	///
	/// Individual routes can still be protected by adding a `#[guard]` attribute.
	pub fn default_public_routes(mut self) -> Self {
		self.default_public = true;
		self
	}

	/// Replaces the entire server configuration with a custom [`MurServerConfig`].
	pub fn configure(mut self, config: MurServerConfig) -> Self {
		self.config = config;
		self
	}

	/// Enables CORS for all origins (`*`).
	pub fn enable_cors(mut self) -> Self {
		self.config = self.config.cors_all();
		self
	}

	/// Enables CORS restricted to the specified `origins`.
	pub fn cors(mut self, origins: Vec<String>) -> Self {
		self.config = self.config.cors(origins);
		self
	}

	/// Disables request/response logging.
	pub fn no_logging(mut self) -> Self {
		self.config = self.config.no_logging();
		self
	}

	/// Registers a callback invoked once when the server finishes starting up.
	pub fn on_startup(mut self, hook: impl Fn() + Send + Sync + 'static) -> Self {
		self.on_startup.push(Box::new(hook));
		self
	}

	/// Registers a callback invoked once when the server is shutting down.
	pub fn on_shutdown(mut self, hook: impl Fn() + Send + Sync + 'static) -> Self {
		self.on_shutdown.push(Box::new(hook));
		self
	}

	/// Adds a [`MurModule`] to the server.
	///
	/// Modules group controllers, services, and imports into reusable units.
	pub fn module(mut self, module: impl MurModule + 'static) -> Self {
		self.modules.push(Box::new(module));
		self
	}

	/// Registers a manually constructed injectable value.
	///
	/// Use this for values that are created outside the DI container (e.g.
	/// configuration objects) and need to be available to DI-enabled types.
	pub fn inject(mut self, inject: impl MurInjectable + 'static) -> Self {
		self.injects.register(inject);
		self
	}

	/// Registers an `Arc`-wrapped injectable value.
	pub fn inject_arc<T: MurInjectable>(mut self, inject: Arc<T>) -> Self {
		self.injects.register_arc(inject);
		self
	}

	/// Registers a singleton service in the global DI container.
	pub fn service<T: MurService + Send + Sync>(mut self, service: T) -> Self {
		self.container.register(service);
		self
	}

	/// Registers an `Arc`-wrapped singleton service in the global DI container.
	pub fn service_arc<T: MurService + Send + Sync>(mut self, service: Arc<T>) -> Self {
		self.container.register_arc(service);
		self
	}

	/// Registers a global guard that applies to every route.
	///
	/// `T` must implement [`MurGuardFactory`] (generated by `#[guard]`).
	pub fn guard<T>(mut self) -> Self
	where
		T: MurGuardFactory + Send + Sync + 'static,
	{
		self.guards.push(Box::new(|injects, container| {
			Box::new(T::__create_factory(injects, container))
		}));
		self
	}

	/// Registers a global transformation pipe applied to handler parameters.
	pub fn pipe<T>(mut self) -> Self
	where
		T: MurPipeFactory + MurPipeDyn + 'static,
	{
		self.pipes.push(Box::new(|injects, container| {
			Box::new(T::__create_factory(injects, container)) as Box<dyn MurPipeDyn>
		}));
		self
	}

	/// Registers a DI-enabled global interceptor.
	///
	/// `T` must implement [`MurInterceptorFactory`] (generated by `#[interceptor]`).
	pub fn interceptor<T>(mut self) -> Self
	where
		T: MurInterceptorFactory + Send + Sync + 'static,
	{
		self.interceptor_factories.push(Box::new(|injects, container| {
			Box::new(T::__create_factory(injects, container)) as Box<dyn MurInterceptor + Send + Sync>
		}));
		self
	}

	/// Registers a pre-built global interceptor without DI.
	pub fn add_global_interceptor(mut self, interceptor: impl MurInterceptor + 'static) -> Self {
		self.interceptor_instances.push(Box::new(interceptor));
		self
	}

	/// Adds a global middleware layer applied to every request before routing.
	pub fn add_middleware(mut self, middleware: impl MurMiddleware + 'static) -> Self {
		self.middleware.push(Box::new(middleware));
		self
	}

	/// Resolves the bind address and finalises the server, returning a
	/// [`MurServerRunner`] ready to call `.run().await` on.
	pub fn bind(self, addr: impl ToSocketAddrs) -> Result<MurServerRunner, std::io::Error> {
		let addr = resolve_addr(addr)?;
		self.bind_addr(addr)
	}

	/// Configures TLS and binds to `addr`.
	///
	/// Requires a valid [`MurTlsConfig`] constructed from PEM certificate and
	/// key material.
	pub fn bind_tls(
		mut self,
		addr: impl ToSocketAddrs,
		tls_config: MurTlsConfig,
	) -> Result<MurServerRunner, std::io::Error> {
		let addr = resolve_addr(addr)?;
		self.config.tls = Some(tls_config);
		self.bind_addr(addr)
	}

	/// Binds to a pre-resolved [`SocketAddr`] and finalises the server.
	pub fn bind_addr(mut self, addr: SocketAddr) -> Result<MurServerRunner, std::io::Error> {
		self.config.addr = addr;
		self.injects.on_init();

		let mut global = MurServiceContainer::new();
		global.merge(self.container);

		let mut runtime = global.clone();
		let mut module_containers: Vec<MurServiceContainer> = Vec::with_capacity(self.modules.len());

		for module in &self.modules {
			module.on_init();

			let mut visible =
				Self::resolve_module_container(module.as_ref(), &self.injects, &global);
			visible.merge(global.clone());

			let local_services = module.services_with_injects(&self.injects, &visible);
			for (tid, svc) in &local_services {
				visible.register_dyn_with_id(*tid, svc.clone());
			}

			module_containers.push(visible);

			let export_ids = module.exports();
			for (tid, svc) in local_services {
				if export_ids.contains(&tid) {
					runtime.register_dyn_with_id(tid, svc);
				}
			}
		}

		let container = Arc::new(runtime);
		let mut router = MurRouter::new(Arc::clone(&container));
		router.default_public = self.default_public;

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

		for (module, module_container) in self.modules.iter().zip(module_containers.iter()) {
			if self.config.enable_logging {
				println!("Loading module: {}", module.name());
			}
			for controller in module.controllers_with_injects(&self.injects, module_container) {
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
