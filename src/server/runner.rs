use super::config::MurServerConfig;
use super::module::MurModule;
use super::router::MurRouter;
use super::security::tls::acceptor::MurTlsAcceptor;
use crate::server::service::MurInjects;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::watch;

// TODO: collect messages and make it default for all methods
// have too many duplicated code
pub struct MurServerRunner {
	pub(crate) router: Arc<MurRouter>,
	pub(crate) config: MurServerConfig,
	pub(crate) modules: Vec<Box<dyn MurModule>>,
	pub(crate) injects: MurInjects,
	pub(crate) on_startup: Vec<Box<dyn Fn() + Send + Sync>>,
	pub(crate) on_shutdown: Vec<Box<dyn Fn() + Send + Sync>>,
	pub(crate) tls_acceptor: Option<MurTlsAcceptor>,
}

impl MurServerRunner {
	pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		for hook in &self.on_startup {
			hook();
		}

		let listener = TcpListener::bind(self.config.addr).await?;
		let is_tls = self.tls_acceptor.is_some();
		let protocol = if is_tls { "https" } else { "http" };

		println!(
			"{} server listening on {}://{}",
			self.config.server_name, protocol, self.config.addr
		);

		if self.tls_acceptor.is_some() {
			if self.config.graceful_shutdown {
				return self.run_tls_with_graceful_shutdown(listener).await;
			} else {
				return self.run_tls_forever(listener).await;
			}
		}

		if self.config.graceful_shutdown {
			self.run_with_graceful_shutdown(listener).await
		} else {
			self.run_forever(listener).await
		}
	}

	async fn run_forever(
		self,
		listener: TcpListener,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		loop {
			let (stream, _remote_addr) = listener.accept().await?;
			let io = TokioIo::new(stream);
			let router = Arc::clone(&self.router);
			let enable_logging = self.config.enable_logging;

			tokio::task::spawn(async move {
				if enable_logging {
					// TODO: analyze
					// Minimal logging - can be expanded
				}

				let service = service_fn(move |req| {
					let router = Arc::clone(&router);
					async move { router.handle(req).await }
				});

				if let Err(err) = http1::Builder::new()
					.serve_connection(io, service)
					.with_upgrades()
					.await && !err.to_string().contains("connection closed")
				{
					eprintln!("Connection error: {}", err);
				}
			});
		}
	}

	async fn run_tls_forever(
		self,
		listener: TcpListener,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let tls_acceptor = self.tls_acceptor.expect("TLS acceptor should be set");

		loop {
			let (stream, _remote_addr) = listener.accept().await?;
			let router = Arc::clone(&self.router);
			let acceptor = tls_acceptor.clone();

			tokio::task::spawn(async move {
				match acceptor.accept(stream).await {
					Ok(tls_stream) => {
						let io = TokioIo::new(tls_stream);

						let service = service_fn(move |req| {
							let router = Arc::clone(&router);
							async move { router.handle(req).await }
						});

						if let Err(err) = http1::Builder::new()
							.serve_connection(io, service)
							.with_upgrades()
							.await && !err.to_string().contains("connection closed")
						{
							eprintln!("TLS connection error: {}", err);
						}
					}
					Err(e) => {
						eprintln!("TLS handshake error: {}", e);
					}
				}
			});
		}
	}

	async fn run_tls_with_graceful_shutdown(
		self,
		listener: TcpListener,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let tls_acceptor = self
			.tls_acceptor
			.clone()
			.expect("TLS acceptor should be set");
		let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
		let shutdown_timeout = self.config.shutdown_timeout;
		let active_connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));

		tokio::spawn(async move {
			tokio::signal::ctrl_c()
				.await
				.expect("Failed to listen for Ctrl+C");
			println!("\nShutdown signal received, starting graceful shutdown...");
			let _ = shutdown_tx.send(true);
		});

		loop {
			tokio::select! {
				result = listener.accept() => {
					match result {
						Ok((stream, _remote_addr)) => {
							let router = Arc::clone(&self.router);
							let connections = Arc::clone(&active_connections);
							let mut shutdown = shutdown_rx.clone();
							let acceptor = tls_acceptor.clone();

							connections.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
							tokio::task::spawn(async move {
								match acceptor.accept(stream).await {
									Ok(tls_stream) => {
										let io = TokioIo::new(tls_stream);

										let service = service_fn(move |req| {
											let router = Arc::clone(&router);
											async move { router.handle(req).await }
										});

										let conn = http1::Builder::new()
											.serve_connection(io, service)
											.with_upgrades();

										tokio::pin!(conn);

										loop {
											tokio::select! {
												result = conn.as_mut() => {
													if let Err(err) = result && !err.to_string().contains("connection closed") {
														eprintln!("TLS connection error: {}", err);
													}
													break;
												}
												_ = shutdown.changed() => {
													conn.as_mut().graceful_shutdown();
												}
											}
										}
									}
									Err(e) => {
										eprintln!("TLS handshake error: {}", e);
									}
								}

								connections.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
							});
						}
						Err(e) => {
							eprintln!("Accept error: {}", e);
						}
					}
				}
				_ = shutdown_rx.changed() => {
					break;
				}
			}
		}

		println!(
			"Waiting for {} active connection(s) to finish...",
			active_connections.load(std::sync::atomic::Ordering::SeqCst)
		);

		let start = std::time::Instant::now();
		while active_connections.load(std::sync::atomic::Ordering::SeqCst) > 0 {
			if start.elapsed() > shutdown_timeout {
				println!("Shutdown timeout exceeded, forcing shutdown...");
				break;
			}
			tokio::time::sleep(Duration::from_millis(100)).await;
		}

		for hook in &self.on_shutdown {
			hook();
		}

		for module in &self.modules {
			module.on_shutdown();
		}
		self.injects.on_shutdown();

		println!("Server shut down gracefully!");
		Ok(())
	}

	async fn run_with_graceful_shutdown(
		self,
		listener: TcpListener,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
		let shutdown_timeout = self.config.shutdown_timeout;
		let active_connections = Arc::new(std::sync::atomic::AtomicUsize::new(0));

		tokio::spawn(async move {
			tokio::signal::ctrl_c()
				.await
				.expect("Failed to listen for Ctrl+C");
			println!("\nShutdown signal received, starting graceful shutdown...");
			let _ = shutdown_tx.send(true);
		});

		loop {
			tokio::select! {
				result = listener.accept() => {
					match result {
						Ok((stream, _remote_addr)) => {
							let io = TokioIo::new(stream);
							let router = Arc::clone(&self.router);
							let connections = Arc::clone(&active_connections);
							let mut shutdown = shutdown_rx.clone();

							connections.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
							tokio::task::spawn(async move {
								let service = service_fn(move |req| {
									let router = Arc::clone(&router);
									async move { router.handle(req).await }
								});

								let conn = http1::Builder::new()
									.serve_connection(io, service)
									.with_upgrades();

								tokio::pin!(conn);

								loop {
									tokio::select! {
										result = conn.as_mut() => {
											if let Err(err) = result &&!err.to_string().contains("connection closed") {
												eprintln!("Connection error: {}", err);
											}
											break;
										}
										_ = shutdown.changed() => {
											conn.as_mut().graceful_shutdown();
										}
									}
								}

								connections.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
							});
						}
						Err(e) => {
							eprintln!("Accept error: {}", e);
						}
					}
				}
				_ = shutdown_rx.changed() => {
					break;
				}
			}
		}

		println!(
			"Waiting for {} active connection(s) to finish...",
			active_connections.load(std::sync::atomic::Ordering::SeqCst)
		);

		let start = std::time::Instant::now();
		while active_connections.load(std::sync::atomic::Ordering::SeqCst) > 0 {
			if start.elapsed() > shutdown_timeout {
				println!("Shutdown timeout exceeded, forcing shutdown...");
				break;
			}
			tokio::time::sleep(Duration::from_millis(100)).await;
		}

		for hook in &self.on_shutdown {
			hook();
		}

		for module in &self.modules {
			module.on_shutdown();
		}
		self.injects.on_shutdown();

		println!("Server shut down gracefully!");
		Ok(())
	}

	pub async fn run_until<F>(
		self,
		shutdown_signal: F,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
	where
		F: Future<Output = ()> + Send + 'static,
	{
		for hook in &self.on_startup {
			hook();
		}

		let listener = TcpListener::bind(self.config.addr).await?;
		let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

		println!(
			"{} server listening on http://{}",
			self.config.server_name, self.config.addr
		);

		tokio::spawn(async move {
			shutdown_signal.await;
			let _ = shutdown_tx.send(true);
		});

		loop {
			tokio::select! {
				result = listener.accept() => {
					match result {
						Ok((stream, _)) => {
							let io = TokioIo::new(stream);
							let router = Arc::clone(&self.router);

							tokio::task::spawn(async move {
								let service = service_fn(move |req| {
									let router = Arc::clone(&router);
									async move { router.handle(req).await }
								});

								let _ = http1::Builder::new()
									.serve_connection(io, service)
									.with_upgrades()
									.await;
							});
						}
						Err(e) => {
							eprintln!("Accept error: {}", e);
						}
					}
				}
				_ = shutdown_rx.changed() => {
					break;
				}
			}
		}

		for hook in &self.on_shutdown {
			hook();
		}

		for module in &self.modules {
			module.on_shutdown();
		}
		self.injects.on_shutdown();

		println!("Server shut down");
		Ok(())
	}

	pub fn addr(&self) -> SocketAddr {
		self.config.addr
	}

	pub fn router(&self) -> &MurRouter {
		&self.router
	}

	pub fn config(&self) -> &MurServerConfig {
		&self.config
	}
}
