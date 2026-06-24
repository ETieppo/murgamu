use super::MurService;
use crate::server::provider::MurProvider;
use crate::server::provider::MurProviderScope;
use crate::server::error::MurError;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// The dependency injection container for Murgamu services.
///
/// `MurServiceContainer` stores and resolves services by their concrete type.
/// It supports three lifetime scopes (see [`MurProviderScope`]):
///
/// - **Singleton** — one shared instance for the entire application lifetime.
/// - **Transient** — a new instance created on every resolution.
/// - **Request** — one instance per HTTP request (cleared after the response).
///
/// Services are registered during server construction and then injected
/// automatically into controllers, guards, interceptors, and other services
/// that declare them as dependencies.
///
/// # Direct access in handlers
///
/// ```rust,ignore
/// #[get("/status")]
/// async fn status(&self, container: Arc<MurServiceContainer>) -> MurRes {
///     let db = container.get_required::<DatabaseService>();
///     mur_json!(serde_json::json!({ "connected": db.is_connected() }))
/// }
/// ```
pub struct MurServiceContainer {
	pub(crate) services: HashMap<TypeId, Arc<dyn MurService>>,
	factories: HashMap<TypeId, Arc<dyn Fn() -> Arc<dyn MurService> + Send + Sync>>,
	request_services: RwLock<HashMap<TypeId, Arc<dyn MurService>>>,
	provider_scopes: HashMap<TypeId, MurProviderScope>,
	aliases: HashMap<TypeId, TypeId>,
}

impl MurServiceContainer {
	/// Creates an empty container.
	#[inline]
	pub fn new() -> Self {
		Self {
			services: HashMap::with_capacity(32),
			factories: HashMap::with_capacity(8),
			request_services: RwLock::new(HashMap::with_capacity(4)),
			provider_scopes: HashMap::with_capacity(32),
			aliases: HashMap::with_capacity(8),
		}
	}

	/// Registers a service as a singleton.
	pub fn register<T: MurService>(&mut self, service: T) {
		let type_id = TypeId::of::<T>();
		self.services.insert(type_id, Arc::new(service));
		self.provider_scopes
			.insert(type_id, MurProviderScope::Singleton);
	}

	/// Registers an already-boxed singleton service.
	pub fn register_arc<T: MurService>(&mut self, service: Arc<T>) {
		let type_id = TypeId::of::<T>();
		self.services
			.insert(type_id, service as Arc<dyn MurService>);
		self.provider_scopes
			.insert(type_id, MurProviderScope::Singleton);
	}

	/// Registers a type-erased singleton, using the concrete type id from `as_any`.
	pub fn register_dyn_service(&mut self, service: Arc<dyn MurService>) {
		let type_id = (*service).as_any().type_id();
		self.services.insert(type_id, service);
		self.provider_scopes
			.insert(type_id, MurProviderScope::Singleton);
	}

	/// Registers a service with an explicit [`MurProviderScope`].
	pub fn register_with_scope<T: MurService>(&mut self, service: T, scope: MurProviderScope) {
		let type_id = TypeId::of::<T>();
		self.services.insert(type_id, Arc::new(service));
		self.provider_scopes.insert(type_id, scope);
	}

	/// Registers a factory closure that creates a new `T` on every resolution (transient scope).
	pub fn register_factory<T: MurService>(
		&mut self,
		factory: impl Fn() -> T + Send + Sync + 'static,
	) {
		let type_id = TypeId::of::<T>();
		self.factories.insert(
			type_id,
			Arc::new(move || Arc::new(factory()) as Arc<dyn MurService>),
		);
		self.provider_scopes
			.insert(type_id, MurProviderScope::Transient);
	}

	/// Registers a service produced by a [`MurProvider`].
	pub fn register_provider<P: MurProvider>(&mut self, provider: P)
	where
		P::Service: 'static,
	{
		let service = provider.provide(self);
		let type_id = TypeId::of::<P::Service>();
		self.services
			.insert(type_id, service as Arc<dyn MurService>);
		self.provider_scopes.insert(type_id, provider.scope());
	}

	/// Creates a type alias so that lookups for `Interface` resolve to `Implementation`.
	pub fn register_alias<Interface: ?Sized + 'static, Implementation: 'static>(&mut self) {
		self.aliases
			.insert(TypeId::of::<Interface>(), TypeId::of::<Implementation>());
	}

	/// Internal method used by the module system to register services by type id.
	pub fn register_dyn_with_id(&mut self, type_id: TypeId, service: Arc<dyn MurService>) {
		self.services.insert(type_id, service);
		self.provider_scopes
			.insert(type_id, MurProviderScope::Singleton);
	}

	/// Resolves the service of type `T`, returning `None` if it is not registered.
	#[inline]
	pub fn get<T: MurService>(&self) -> Option<Arc<T>> {
		let type_id = TypeId::of::<T>();
		let resolved_type_id = self.aliases.get(&type_id).copied().unwrap_or(type_id);

		if let Some(service) = self.services.get(&resolved_type_id) {
			return self.downcast_arc(service);
		}

		// Request-scoped services are stored separately via `set_request_service`,
		// which takes `&self` and therefore cannot record a scope in the
		// (non-interior-mutable) `provider_scopes` map. Always consult the
		// request store so such services are actually resolvable.
		if let Ok(request_services) = self.request_services.read()
			&& let Some(service) = request_services.get(&resolved_type_id)
		{
			return self.downcast_arc(service);
		}

		if let Some(MurProviderScope::Transient) = self.provider_scopes.get(&resolved_type_id)
			&& let Some(factory) = self.factories.get(&resolved_type_id)
		{
			let service = factory();
			return self.downcast_arc(&service);
		}

		None
	}

	/// Resolves the service of type `T`, panicking if it is not registered.
	pub fn get_required<T: MurService>(&self) -> Arc<T> {
		self.get::<T>()
			.unwrap_or_else(|| panic!("Required service not found: {}", std::any::type_name::<T>()))
	}

	/// Resolves the service of type `T`, returning an error if it is not registered.
	pub fn try_get_required<T: MurService>(&self) -> Result<Arc<T>, MurError> {
		self.get::<T>().ok_or_else(|| {
			MurError::Internal(format!(
				"Required service not found: {}",
				std::any::type_name::<T>()
			))
		})
	}

	/// Returns `true` if a service of type `T` is registered.
	#[inline]
	pub fn has<T: MurService>(&self) -> bool {
		let type_id = TypeId::of::<T>();
		let resolved_type_id = self.aliases.get(&type_id).copied().unwrap_or(type_id);
		self.services.contains_key(&resolved_type_id)
			|| self.factories.contains_key(&resolved_type_id)
	}

	/// Returns the [`MurProviderScope`] of the service of type `T`, if registered.
	#[inline]
	pub fn scope_of<T: MurService>(&self) -> Option<MurProviderScope> {
		let type_id = TypeId::of::<T>();
		let resolved_type_id = self.aliases.get(&type_id).copied().unwrap_or(type_id);
		self.provider_scopes.get(&resolved_type_id).copied()
	}

	/// Stores a request-scoped service.
	///
	/// The service is available for the duration of the current request and
	/// is cleared by [`clear_request_services`](Self::clear_request_services).
	pub fn set_request_service<T: MurService>(&self, service: T) {
		let type_id = TypeId::of::<T>();
		if let Ok(mut request_services) = self.request_services.write() {
			request_services.insert(type_id, Arc::new(service));
		}
	}

	/// Removes all request-scoped services from the container.
	pub fn clear_request_services(&self) {
		if let Ok(mut request_services) = self.request_services.write() {
			request_services.clear();
		}
	}

	/// Returns the type IDs of all registered singleton services.
	pub fn registered_services(&self) -> Vec<TypeId> {
		self.services.keys().copied().collect()
	}

	/// Returns the total number of registered services and factories.
	#[inline]
	pub fn len(&self) -> usize {
		self.services.len() + self.factories.len()
	}

	/// Returns `true` if no services or factories are registered.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.services.is_empty() && self.factories.is_empty()
	}

	#[inline]
	fn downcast_arc<T: MurService>(&self, service: &Arc<dyn MurService>) -> Option<Arc<T>> {
		if service.as_any().type_id() != TypeId::of::<T>() {
			debug_assert!(false, "TypeId mismatch in downcast_arc — framework bug");
			return None;
		}
		let ptr = Arc::as_ptr(service) as *const T;
		// SAFETY: type_id() confirmed the concrete type behind the fat pointer is T.
		// Arc::as_ptr on Arc<dyn Trait> (created from Arc<T>) returns the data
		// pointer, which is *const T. Incrementing the strong count before from_raw
		// keeps the allocation alive for the lifetime of the returned Arc.
		unsafe {
			Arc::increment_strong_count(ptr);
			Some(Arc::from_raw(ptr))
		}
	}

	/// Merges all services, factories, scopes, and aliases from `other` into `self`.
	pub fn merge(&mut self, other: MurServiceContainer) {
		self.services.extend(other.services);
		self.factories.extend(other.factories);
		self.provider_scopes.extend(other.provider_scopes);
		self.aliases.extend(other.aliases);
	}

	/// Creates a child container that inherits all services but has its own
	/// request-scoped service store.
	pub fn create_child(&self) -> MurServiceContainer {
		MurServiceContainer {
			services: self.services.clone(),
			factories: self.factories.clone(),
			request_services: RwLock::new(HashMap::with_capacity(4)),
			provider_scopes: self.provider_scopes.clone(),
			aliases: self.aliases.clone(),
		}
	}
}

impl Default for MurServiceContainer {
	fn default() -> Self {
		Self::new()
	}
}

impl Clone for MurServiceContainer {
	fn clone(&self) -> Self {
		Self {
			services: self.services.clone(),
			factories: self.factories.clone(),
			request_services: RwLock::new(
				self.request_services
					.read()
					.map(|r| r.clone())
					.unwrap_or_default(),
			),
			provider_scopes: self.provider_scopes.clone(),
			aliases: self.aliases.clone(),
		}
	}
}

impl std::fmt::Debug for MurServiceContainer {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurServiceContainer")
			.field("services_count", &self.services.len())
			.field("factories_count", &self.factories.len())
			.field("aliases_count", &self.aliases.len())
			.finish()
	}
}

#[cfg(test)]
mod tests {
	use std::any::Any;
	use super::*;

	struct TestService {
		value: i32,
	}

	impl MurService for TestService {
		fn as_any(&self) -> &dyn Any {
			self
		}
	}

	#[test]
	fn test_register_and_get() {
		let mut container = MurServiceContainer::new();
		container.register(TestService { value: 42 });

		let service = container.get::<TestService>().unwrap();
		assert_eq!(service.value, 42);
	}

	#[test]
	fn test_has() {
		let mut container = MurServiceContainer::new();
		assert!(!container.has::<TestService>());

		container.register(TestService { value: 42 });
		assert!(container.has::<TestService>());
	}

	#[test]
	fn test_downcast_arc_shares_same_allocation() {
		let mut container = MurServiceContainer::new();
		container.register(TestService { value: 42 });

		let service1 = container.get::<TestService>().unwrap();
		let service2 = container.get::<TestService>().unwrap();

		assert_eq!(Arc::as_ptr(&service1), Arc::as_ptr(&service2));
		assert_eq!(Arc::strong_count(&service1), 3);
	}

	struct OtherService {
		tag: &'static str,
	}

	impl MurService for OtherService {
		fn as_any(&self) -> &dyn Any {
			self
		}
	}

	#[test]
	fn unregistered_service_returns_none() {
		let container = MurServiceContainer::new();
		assert!(container.get::<TestService>().is_none());
		assert!(!container.has::<TestService>());
	}

	#[test]
	fn transient_factory_creates_new_instances() {
		let mut container = MurServiceContainer::new();
		let counter = std::sync::atomic::AtomicI32::new(0);
		container.register_factory(move || {
			let v = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
			TestService { value: v }
		});

		let a = container.get::<TestService>().unwrap();
		let b = container.get::<TestService>().unwrap();
		// Different allocations and increasing values prove fresh construction.
		assert_ne!(Arc::as_ptr(&a), Arc::as_ptr(&b));
		assert_ne!(a.value, b.value);
		assert!(container.has::<TestService>());
		assert_eq!(container.scope_of::<TestService>(), Some(MurProviderScope::Transient));
	}

	#[test]
	fn request_scoped_service_is_resolvable_and_clearable() {
		// Regression: services stored via `set_request_service` used to be
		// unreachable from `get` because no scope was recorded.
		let container = MurServiceContainer::new();
		container.set_request_service(TestService { value: 7 });

		let resolved = container.get::<TestService>();
		assert!(resolved.is_some(), "request-scoped service must resolve");
		assert_eq!(resolved.unwrap().value, 7);

		container.clear_request_services();
		assert!(container.get::<TestService>().is_none());
	}

	#[test]
	fn alias_resolves_to_implementation() {
		let mut container = MurServiceContainer::new();
		container.register(OtherService { tag: "impl" });
		// Alias TestService -> OtherService's type id.
		container.register_alias::<TestService, OtherService>();

		// Looking up via the alias key resolves to the implementation type.
		assert!(container.has::<TestService>());
		let resolved = container.get::<OtherService>().unwrap();
		assert_eq!(resolved.tag, "impl");
	}

	#[test]
	fn merge_combines_services() {
		let mut a = MurServiceContainer::new();
		a.register(TestService { value: 1 });
		let mut b = MurServiceContainer::new();
		b.register(OtherService { tag: "b" });

		a.merge(b);
		assert!(a.get::<TestService>().is_some());
		assert!(a.get::<OtherService>().is_some());
	}

	#[test]
	fn child_container_has_isolated_request_store() {
		let mut parent = MurServiceContainer::new();
		parent.register(TestService { value: 5 });
		let child = parent.create_child();

		// Child sees inherited singletons.
		assert!(child.get::<TestService>().is_some());

		// Request services set on the child do not leak to the parent.
		child.set_request_service(OtherService { tag: "child-only" });
		assert!(child.get::<OtherService>().is_some());
		assert!(parent.get::<OtherService>().is_none());
	}
}
