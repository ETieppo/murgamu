use super::container::MurServiceContainer;
use super::MurService;
use crate::server::provider::MurProvider;
use std::sync::Arc;

pub struct MurServiceContainerBuilder {
	container: MurServiceContainer,
}

impl MurServiceContainerBuilder {
	pub fn new() -> Self {
		Self {
			container: MurServiceContainer::new(),
		}
	}

	pub fn singleton<T: MurService>(mut self, service: T) -> Self {
		self.container.register(service);
		self
	}

	pub fn singleton_arc<T: MurService>(mut self, service: Arc<T>) -> Self {
		self.container.register_arc(service);
		self
	}

	pub fn transient<T: MurService>(
		mut self,
		factory: impl Fn() -> T + Send + Sync + 'static,
	) -> Self {
		self.container.register_factory(factory);
		self
	}

	pub fn provider<P: MurProvider>(mut self, provider: P) -> Self
	where
		P::Service: 'static,
	{
		self.container.register_provider(provider);
		self
	}

	pub fn alias<Interface: ?Sized + 'static, Implementation: 'static>(mut self) -> Self {
		self.container.register_alias::<Interface, Implementation>();
		self
	}

	pub fn build(self) -> MurServiceContainer {
		self.container
	}
}

impl Default for MurServiceContainerBuilder {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::any::Any;

	struct TestService {
		value: i32,
	}

	impl MurService for TestService {
		fn as_any(&self) -> &dyn Any {
			self
		}
	}

	#[test]
	fn test_builder() {
		let container = MurServiceContainerBuilder::new()
			.singleton(TestService { value: 100 })
			.build();

		let service = container.get::<TestService>().unwrap();
		assert_eq!(service.value, 100);
	}
}
