use super::MurService;
use crate::server::provider::MurProvider;
use crate::server::provider::MurProviderScope;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct MurServiceContainer {
	pub(crate) services: HashMap<TypeId, Arc<dyn MurService>>,
	factories: HashMap<TypeId, Arc<dyn Fn() -> Arc<dyn MurService> + Send + Sync>>,
	request_services: RwLock<HashMap<TypeId, Arc<dyn MurService>>>,
	provider_scopes: HashMap<TypeId, MurProviderScope>,
	aliases: HashMap<TypeId, TypeId>,
}

impl MurServiceContainer {
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

	pub fn register<T: MurService>(&mut self, service: T) {
		let type_id = TypeId::of::<T>();
		self.services.insert(type_id, Arc::new(service));
		self.provider_scopes
			.insert(type_id, MurProviderScope::Singleton);
	}

	pub fn register_arc<T: MurService>(&mut self, service: Arc<T>) {
		let type_id = TypeId::of::<T>();
		self.services
			.insert(type_id, service as Arc<dyn MurService>);
		self.provider_scopes
			.insert(type_id, MurProviderScope::Singleton);
	}

	pub fn register_dyn_service(&mut self, service: Arc<dyn MurService>) {
		let type_id = (*service).as_any().type_id();
		self.services.insert(type_id, service);
		self.provider_scopes
			.insert(type_id, MurProviderScope::Singleton);
	}

	pub fn register_with_scope<T: MurService>(&mut self, service: T, scope: MurProviderScope) {
		let type_id = TypeId::of::<T>();
		self.services.insert(type_id, Arc::new(service));
		self.provider_scopes.insert(type_id, scope);
	}

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

	pub fn register_alias<Interface: ?Sized + 'static, Implementation: 'static>(&mut self) {
		self.aliases
			.insert(TypeId::of::<Interface>(), TypeId::of::<Implementation>());
	}

	pub fn register_dyn_with_id(&mut self, type_id: TypeId, service: Arc<dyn MurService>) {
		self.services.insert(type_id, service);
		self.provider_scopes
			.insert(type_id, MurProviderScope::Singleton);
	}

	#[inline]
	pub fn get<T: MurService>(&self) -> Option<Arc<T>> {
		let type_id = TypeId::of::<T>();
		let resolved_type_id = self.aliases.get(&type_id).copied().unwrap_or(type_id);

		if let Some(service) = self.services.get(&resolved_type_id) {
			return self.downcast_arc(service);
		}

		match self.provider_scopes.get(&resolved_type_id) {
			Some(MurProviderScope::Transient) => {
				if let Some(factory) = self.factories.get(&resolved_type_id) {
					let service = factory();
					return self.downcast_arc(&service);
				}
			}
			Some(MurProviderScope::Request) => {
				if let Ok(request_services) = self.request_services.read()
					&& let Some(service) = request_services.get(&resolved_type_id)
				{
					return self.downcast_arc(service);
				}
			}
			_ => {}
		}

		None
	}

	pub fn get_required<T: MurService>(&self) -> Arc<T> {
		self.get::<T>()
			.unwrap_or_else(|| panic!("Required service not found: {}", std::any::type_name::<T>()))
	}

	#[inline]
	pub fn has<T: MurService>(&self) -> bool {
		let type_id = TypeId::of::<T>();
		let resolved_type_id = self.aliases.get(&type_id).copied().unwrap_or(type_id);
		self.services.contains_key(&resolved_type_id)
			|| self.factories.contains_key(&resolved_type_id)
	}

	#[inline]
	pub fn scope_of<T: MurService>(&self) -> Option<MurProviderScope> {
		let type_id = TypeId::of::<T>();
		let resolved_type_id = self.aliases.get(&type_id).copied().unwrap_or(type_id);
		self.provider_scopes.get(&resolved_type_id).copied()
	}

	pub fn set_request_service<T: MurService>(&self, service: T) {
		let type_id = TypeId::of::<T>();
		if let Ok(mut request_services) = self.request_services.write() {
			request_services.insert(type_id, Arc::new(service));
		}
	}

	pub fn clear_request_services(&self) {
		if let Ok(mut request_services) = self.request_services.write() {
			request_services.clear();
		}
	}

	pub fn registered_services(&self) -> Vec<TypeId> {
		self.services.keys().copied().collect()
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.services.len() + self.factories.len()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.services.is_empty() && self.factories.is_empty()
	}

	#[inline]
	fn downcast_arc<T: MurService>(&self, service: &Arc<dyn MurService>) -> Option<Arc<T>> {
		let any_ref: &dyn Any = service.as_any();
		if any_ref.downcast_ref::<T>().is_some() {
			let ptr = Arc::as_ptr(service) as *const T;

			unsafe {
				Arc::increment_strong_count(ptr);
				Some(Arc::from_raw(ptr))
			}
		} else {
			None
		}
	}

	pub fn merge(&mut self, other: MurServiceContainer) {
		self.services.extend(other.services);
		self.factories.extend(other.factories);
		self.provider_scopes.extend(other.provider_scopes);
		self.aliases.extend(other.aliases);
	}

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
}
