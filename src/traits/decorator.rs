use std::collections::HashMap;

pub trait MurDecorator: Send + Sync + 'static {
	fn metadata(&self) -> HashMap<String, String>;

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}
