use crate::config::MurConfigResult;
use std::collections::HashMap;

pub trait MurConfigSource: Send + Sync {
	fn load(&self) -> MurConfigResult<HashMap<String, String>>;
	fn name(&self) -> &str;

	fn priority(&self) -> i32 {
		0
	}

	fn is_available(&self) -> bool {
		true
	}
}
