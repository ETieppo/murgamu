#[derive(Debug, Clone)]
pub struct EnvServerDefaults {
	pub host: String,
	pub port: u16,
	pub enable_cors: bool,
	pub enable_logging: bool,
	pub log_requests: bool,
	pub graceful_shutdown: bool,
	pub keep_alive_secs: u64,
}

impl EnvServerDefaults {
	pub fn addr(&self) -> String {
		format!("{}:{}", self.host, self.port)
	}
}
