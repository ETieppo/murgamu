use crate::{container::core::MurServiceContainer, types::MurRouteHandler};
use std::sync::Arc;

pub trait MurRepository: Send + Sync + 'static {
	fn routes(
		self: Arc<Self>,
		container: Arc<MurServiceContainer>,
	) -> Vec<(String, String, MurRouteHandler)>;

	fn base_path(&self) -> &str {
		"/"
	}

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}

pub trait MurCloneRepository: MurRepository + Clone {}
impl<T: MurRepository + Clone> MurCloneRepository for T {}
