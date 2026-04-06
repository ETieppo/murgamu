use crate::MurServiceContainer;
use crate::server::controller::MurController;
use crate::server::service::MurInjects;
use crate::server::service::MurService;
use std::any::TypeId;
use std::sync::Arc;

pub trait MurModuleConfig {
	type Options;
	fn with_options(options: Self::Options) -> Self;
}

pub trait MurModule: Send + Sync + 'static {
	fn controllers(&self) -> Vec<Arc<dyn MurController>> {
		Vec::new()
	}

	fn controllers_with_injects(
		&self,
		_injects: &MurInjects,
		_container: &MurServiceContainer,
	) -> Vec<Arc<dyn MurController>> {
		self.controllers()
	}

	fn services(&self) -> Vec<(TypeId, Arc<dyn MurService>)> {
		Vec::new()
	}

	fn services_with_injects(
		&self,
		_injects: &MurInjects,
		_container: &MurServiceContainer,
	) -> Vec<(TypeId, Arc<dyn MurService>)> {
		self.services()
	}

	fn name(&self) -> &str;
	fn exports(&self) -> Vec<TypeId>;
	fn imports(&self) -> Vec<Arc<dyn MurModule>>;
	fn on_init(&self) {}
	fn on_shutdown(&self) {}
}
