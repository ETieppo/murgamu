use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MurSseConfig {
	pub keep_alive: bool,
	pub keep_alive_interval: Duration,
	pub retry_interval: Option<Duration>,
	pub headers: HashMap<String, String>,
}

impl Default for MurSseConfig {
	fn default() -> Self {
		Self {
			keep_alive: true,
			keep_alive_interval: Duration::from_secs(30),
			retry_interval: Some(Duration::from_secs(3)),
			headers: HashMap::new(),
		}
	}
}
