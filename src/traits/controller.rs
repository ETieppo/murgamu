use crate::{
	container::core::MurServiceContainer,
	types::{MurRouteDefinition},
};
use std::sync::Arc;

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
