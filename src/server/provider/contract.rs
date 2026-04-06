use crate::server::service::MurService;
use crate::server::service::MurServiceContainer;
use std::sync::Arc;

pub trait MurProvider: Send + Sync + 'static {
	type Service: MurService;

	fn provide(&self, container: &MurServiceContainer) -> Arc<Self::Service>;

	fn scope(&self) -> MurProviderScope {
		MurProviderScope::Singleton
	}

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MurProviderScope {
	Singleton,
	Request,
	Transient,
}
