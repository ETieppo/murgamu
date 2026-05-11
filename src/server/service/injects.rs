use super::MurInjectable;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct MurInjects {
	items: HashMap<TypeId, Arc<dyn MurInjectable + Send + Sync>>,
}

impl MurInjects {
	#[inline]
	pub fn new() -> Self {
		Self {
			items: HashMap::with_capacity(8),
		}
	}

	pub fn register<T: MurInjectable>(&mut self, inject: T) {
		let type_id = TypeId::of::<T>();
		self.items.insert(type_id, Arc::new(inject));
	}

	pub fn register_arc<T: MurInjectable>(&mut self, inject: Arc<T>) {
		let type_id = TypeId::of::<T>();
		self.items.insert(type_id, inject as Arc<dyn MurInjectable>);
	}

	pub fn register_dyn(&mut self, inject: Arc<dyn MurInjectable>) {
		let type_id = inject.as_any().type_id();
		self.items.insert(type_id, inject);
	}

	#[inline]
	pub fn get<T: MurInjectable + Send + Sync>(&self) -> Option<Arc<T>> {
		let type_id = TypeId::of::<T>();
		self.items
			.get(&type_id)
			.and_then(|inject| self.downcast_arc(inject))
	}

	pub fn get_required<T: MurInjectable>(&self) -> Arc<T> {
		self.get::<T>()
			.unwrap_or_else(|| panic!("Required inject not found: {}", std::any::type_name::<T>()))
	}

	#[inline]
	pub fn has<T: MurInjectable>(&self) -> bool {
		let type_id = TypeId::of::<T>();
		self.items.contains_key(&type_id)
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.items.len()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.items.is_empty()
	}

	pub fn on_init(&self) {
		for inject in self.items.values() {
			inject.on_init();
		}
	}

	pub fn on_shutdown(&self) {
		for inject in self.items.values() {
			inject.on_shutdown();
		}
	}

	#[inline]
	fn downcast_arc<T: MurInjectable>(
		&self,
		inject: &Arc<dyn MurInjectable + Send + Sync>,
	) -> Option<Arc<T>> {
		if inject.as_any().type_id() != TypeId::of::<T>() {
			return None;
		}
		let ptr = Arc::as_ptr(inject) as *const T;
		// SAFETY: type_id() confirmed the concrete type behind the fat pointer is T.
		// Arc::as_ptr on Arc<dyn Trait> (created from Arc<T>) returns the data
		// pointer, which is *const T. Incrementing the strong count before from_raw
		// keeps the allocation alive for the lifetime of the returned Arc.
		unsafe {
			Arc::increment_strong_count(ptr);
			Some(Arc::from_raw(ptr))
		}
	}
}

impl std::fmt::Debug for MurInjects {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurInjects")
			.field("injects_count", &self.items.len())
			.finish()
	}
}
