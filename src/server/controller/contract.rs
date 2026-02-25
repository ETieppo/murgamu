use crate::MurInjects;
use crate::server::router::MurRouteDefinition;
use crate::server::service::MurServiceContainer;
use std::sync::Arc;

pub trait IntoController {
	fn into_controller(self) -> Arc<dyn MurController>;
}

impl<T: MurController + 'static> IntoController for T {
	fn into_controller(self) -> Arc<dyn MurController> {
		Arc::new(self)
	}
}

impl IntoController for Arc<dyn MurController> {
	fn into_controller(self) -> Arc<dyn MurController> {
		self
	}
}

pub trait MurControllerFactory: Sized + MurController + Sync + Send {
	fn create(injects: &MurInjects, container: &MurServiceContainer) -> Self;
}

pub trait MurController: Send + Sync + 'static {
	fn routes(self: Arc<Self>, container: Arc<MurServiceContainer>) -> Vec<MurRouteDefinition>;

	fn base_path(&self) -> &str {
		"/"
	}

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}

pub trait MurCloneController: MurController + Clone {}
impl<T: MurController + Clone> MurCloneController for T {}
