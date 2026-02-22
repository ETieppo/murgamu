use super::MurInjects;
use std::any::Any;

pub trait MurService: Send + Sync + 'static {
	fn as_any(&self) -> &dyn Any;
	fn on_init(&self) {}
	fn on_shutdown(&self) {}
}

pub trait MurServiceFactory: MurService {
	fn create(injects: &MurInjects) -> Self;
}

pub trait MurInjectable: Send + Sync + 'static {
	fn as_any(&self) -> &dyn Any;
	fn on_init(&self) {}
	fn on_shutdown(&self) {}
}
