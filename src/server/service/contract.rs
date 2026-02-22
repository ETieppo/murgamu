use super::MurInjects;
use crate::MurServiceContainer;
use std::any::Any;

pub trait MurService: Send + Sync + 'static {
	fn as_any(&self) -> &dyn Any;
	fn on_init(&self) {}
	fn on_shutdown(&self) {}
}

pub trait MurServiceFactory: MurService {
	fn create(injects: &MurInjects, _container: &MurServiceContainer) -> Self;
}

pub trait MurInjectable: Send + Sync + 'static {
	fn as_any(&self) -> &dyn Any;
	fn on_init(&self) {}
	fn on_shutdown(&self) {}
}
