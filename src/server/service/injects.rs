use super::MurInjectable;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct MurInjects {
	items: HashMap<TypeId, Arc<dyn MurInjectable>>,
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
	pub fn get<T: MurInjectable>(&self) -> Option<Arc<T>> {
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
	fn downcast_arc<T: MurInjectable>(&self, inject: &Arc<dyn MurInjectable>) -> Option<Arc<T>> {
		let any_ref: &dyn Any = inject.as_any();
		if any_ref.downcast_ref::<T>().is_some() {
			let ptr = Arc::as_ptr(inject) as *const T;

			unsafe {
				Arc::increment_strong_count(ptr);
				Some(Arc::from_raw(ptr))
			}
		} else {
			None
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
