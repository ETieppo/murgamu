use crate::security::tls::config::MurTlsConfig;
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MurServerConfig {
	pub addr: SocketAddr,
	pub keep_alive_timeout: Option<Duration>,
	pub body_limit: usize,
	pub http2: bool,
	pub graceful_shutdown: bool,
	pub shutdown_timeout: Duration,
	pub enable_logging: bool,
	pub server_name: String,
	pub enable_cors: bool,
	pub cors_origins: Vec<String>,
	pub tls: Option<MurTlsConfig>,
}

impl Default for MurServerConfig {
	fn default() -> Self {
		Self {
			addr: "127.0.0.1:3000".parse().unwrap(),
			keep_alive_timeout: Some(Duration::from_secs(60)),
			body_limit: 2 * 1024 * 1024, // 2MB
			http2: false,
			graceful_shutdown: true,
			shutdown_timeout: Duration::from_secs(30),
			enable_logging: true,
			server_name: String::from("Murgamu"),
			enable_cors: false,
			cors_origins: vec![String::from("*")],
			tls: None,
		}
	}
}

impl MurServerConfig {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn addr(mut self, addr: impl Into<SocketAddr>) -> Self {
		self.addr = addr.into();
		self
	}

	pub fn keep_alive(mut self, timeout: Duration) -> Self {
		self.keep_alive_timeout = Some(timeout);
		self
	}

	pub fn no_keep_alive(mut self) -> Self {
		self.keep_alive_timeout = None;
		self
	}

	pub fn body_limit(mut self, limit: usize) -> Self {
		self.body_limit = limit;
		self
	}

	pub fn http2(mut self) -> Self {
		self.http2 = true;
		self
	}

	pub fn no_graceful_shutdown(mut self) -> Self {
		self.graceful_shutdown = false;
		self
	}

	pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
		self.shutdown_timeout = timeout;
		self
	}

	pub fn no_logging(mut self) -> Self {
		self.enable_logging = false;
		self
	}

	pub fn server_name(mut self, name: impl Into<String>) -> Self {
		self.server_name = name.into();
		self
	}

	pub fn cors(mut self, origins: Vec<String>) -> Self {
		self.enable_cors = true;
		self.cors_origins = origins;
		self
	}

	pub fn cors_all(mut self) -> Self {
		self.enable_cors = true;
		self.cors_origins = vec![String::from("*")];
		self
	}

	pub fn tls(mut self, config: MurTlsConfig) -> Self {
		self.tls = Some(config);
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_config_defaults() {
		let config = MurServerConfig::default();
		assert_eq!(config.addr.to_string(), "127.0.0.1:3000");
		assert_eq!(config.body_limit, 2 * 1024 * 1024);
		assert!(config.graceful_shutdown);
	}

	#[test]
	fn test_config_builder() {
		let config = MurServerConfig::new()
			.addr("0.0.0.0:8080".parse::<SocketAddr>().unwrap())
			.body_limit(5 * 1024 * 1024)
			.no_graceful_shutdown()
			.cors_all();

		assert_eq!(config.addr.to_string(), "0.0.0.0:8080");
		assert_eq!(config.body_limit, 5 * 1024 * 1024);
		assert!(!config.graceful_shutdown);
		assert!(config.enable_cors);
	}
}
