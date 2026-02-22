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
	fn name(&self) -> &str;

	fn controllers(&self) -> Vec<Arc<dyn MurController>> {
		Vec::new()
	}

	fn controllers_with_injects(&self, _injects: &MurInjects) -> Vec<Arc<dyn MurController>> {
		self.controllers()
	}

	fn services(&self) -> Vec<(TypeId, Arc<dyn MurService>)> {
		Vec::new()
	}

	fn services_with_injects(&self, _injects: &MurInjects) -> Vec<(TypeId, Arc<dyn MurService>)> {
		self.services()
	}

	fn on_init(&self) {}

	fn on_shutdown(&self) {}
}
