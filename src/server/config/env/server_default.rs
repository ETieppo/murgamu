/// Default server configuration values for a specific [`MurEnvProfile`](crate::MurEnvProfile).
///
/// Each environment profile (development, staging, production, etc.) ships
/// with sensible defaults for host, port, CORS, logging, and keep-alive so
/// that a new application works out-of-the-box without manual configuration.
#[derive(Debug, Clone)]
pub struct EnvServerDefaults {
	/// The host address the server binds to (e.g. `"127.0.0.1"` or `"0.0.0.0"`).
	pub host: String,
	/// The TCP port the server listens on.
	pub port: u16,
	/// Whether CORS headers should be included in responses by default.
	pub enable_cors: bool,
	/// Whether request/response logging is enabled.
	pub enable_logging: bool,
	/// Whether individual HTTP request details are logged.
	pub log_requests: bool,
	/// Whether the server waits for in-flight requests to complete before shutting down.
	pub graceful_shutdown: bool,
	/// TCP keep-alive idle timeout in seconds.
	pub keep_alive_secs: u64,
}

impl EnvServerDefaults {
	/// Returns `"host:port"` as a formatted bind address string.
	pub fn addr(&self) -> String {
		format!("{}:{}", self.host, self.port)
	}
}
