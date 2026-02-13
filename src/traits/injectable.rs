use std::any::Any;

pub trait MurInjectable: Send + Sync + 'static {
	fn as_any(&self) -> &dyn Any;
	fn on_init(&self) {}
	fn on_shutdown(&self) {}
}
