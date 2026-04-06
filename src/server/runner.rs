use super::config::MurServerConfig;
use super::module::MurModule;
use super::router::MurRouter;
use super::security::tls::MurTlsAcceptor;
use crate::MurError;
use crate::server::security::limited_body_extraction;
use crate::server::service::MurInjects;
use http::{Request, Response};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::rt::{Read, Write};
use hyper::server::conn::http1;
use hyper::service::{Service, service_fn};
use hyper_util::rt::TokioIo;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;

// ---------------------------------------------------------------------------
// Tipos internos
// ---------------------------------------------------------------------------

/// Controla como o servidor lida com o desligamento.
enum ShutdownMode {
	/// Roda para sempre — Ctrl+C mata o processo diretamente.
	Forever,
	/// Aguarda as conexões ativas terminarem antes de sair.
	Graceful { timeout: Duration },
}

// ---------------------------------------------------------------------------
// MurServerRunner
// ---------------------------------------------------------------------------

pub struct MurServerRunner {
	pub(crate) router: Arc<MurRouter>,
	pub(crate) config: MurServerConfig,
	pub(crate) modules: Vec<Box<dyn MurModule + Send + Sync>>,
	pub(crate) injects: MurInjects,
	pub(crate) on_startup: Vec<Box<dyn Fn() + Send + Sync>>,
	pub(crate) on_shutdown: Vec<Box<dyn Fn() + Send + Sync>>,
	pub(crate) tls_acceptor: Option<MurTlsAcceptor>,
}

impl MurServerRunner {
	// -----------------------------------------------------------------------
	// API pública
	// -----------------------------------------------------------------------

	pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		self.run_with_signal(default_shutdown_signal()).await
	}

	pub async fn run_until<F>(
		self,
		shutdown_signal: F,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
	where
		F: Future<Output = ()> + Send + 'static,
	{
		self.run_with_signal(shutdown_signal).await
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

	// -----------------------------------------------------------------------
	// Núcleo — um único caminho de execução
	// -----------------------------------------------------------------------

	async fn run_with_signal<F>(
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
		let protocol = if self.tls_acceptor.is_some() { "https" } else { "http" };

		println!(
			"{} server listening on {}://{}",
			self.config.server_name, protocol, self.config.addr
		);

		let shutdown_mode = if self.config.graceful_shutdown {
			ShutdownMode::Graceful { timeout: self.config.shutdown_timeout }
		} else {
			ShutdownMode::Forever
		};

		self.accept_loop(listener, shutdown_signal, shutdown_mode).await
	}

	/// Loop central de accept — lida com plain TCP e TLS de forma unificada.
	async fn accept_loop<F>(
		self,
		listener: TcpListener,
		shutdown_signal: F,
		mode: ShutdownMode,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
	where
		F: Future<Output = ()> + Send + 'static,
	{
		let (shutdown_tx, shutdown_rx) = watch::channel(false);
		let active = Arc::new(AtomicUsize::new(0));

		// Tarefa que aguarda o sinal de shutdown (Ctrl+C ou sinal customizado)
		tokio::spawn(async move {
			shutdown_signal.await;
			println!("\nShutdown signal received, starting graceful shutdown...");
			let _ = shutdown_tx.send(true);
		});

		loop {
			// Materializa o future de shutdown antes do select! para evitar
			// "temporary value dropped while borrowed" (E0716).
			let mut changed = shutdown_rx.clone();
			tokio::select! {
				result = listener.accept() => {
					match result {
						Ok((stream, _)) => {
							self.spawn_connection(
								stream,
								Arc::clone(&active),
								shutdown_rx.clone(),
							);
						}
						Err(e) => eprintln!("Accept error: {}", e),
					}
				}
				_ = changed.changed() => break,
			}
		}

		// Graceful: aguarda conexões ativas
		if let ShutdownMode::Graceful { timeout } = mode {
			wait_for_connections(&active, timeout).await;
		}

		self.run_shutdown_hooks();
		println!("Server shut down gracefully!");
		Ok(())
	}

	/// Cria uma task para uma conexão nova, com ou sem TLS.
	fn spawn_connection(
		&self,
		stream: TcpStream,
		active: Arc<AtomicUsize>,
		mut shutdown_rx: watch::Receiver<bool>,
	) {
		let router = Arc::clone(&self.router);
		let limit = self.config.body_limit;
		let tls = self.tls_acceptor.clone();

		active.fetch_add(1, Ordering::Relaxed);

		tokio::spawn(async move {
			match tls {
				Some(acceptor) => match acceptor.accept(stream).await {
					Ok(tls_stream) => {
						serve(TokioIo::new(tls_stream), router, limit, &mut shutdown_rx).await;
					}
					Err(e) => eprintln!("TLS handshake error: {}", e),
				},
				None => {
					serve(TokioIo::new(stream), router, limit, &mut shutdown_rx).await;
				}
			}

			active.fetch_sub(1, Ordering::Relaxed);
		});
	}

	fn run_shutdown_hooks(&self) {
		for hook in &self.on_shutdown {
			hook();
		}
		for module in &self.modules {
			module.on_shutdown();
		}
		self.injects.on_shutdown();
	}
}

// ---------------------------------------------------------------------------
// Funções auxiliares livres
// ---------------------------------------------------------------------------

/// Serve uma única conexão com suporte a graceful shutdown.
/// Funciona com qualquer stream que implemente os bounds do hyper.
async fn serve<I>(
	io: I,
	router: Arc<MurRouter>,
	limit: usize,
	shutdown_rx: &mut watch::Receiver<bool>,
) where
	I: Read + Write + Unpin + Send + 'static,
{
	let service = make_service(router, limit);
	let conn = http1::Builder::new()
		.serve_connection(io, service)
		.with_upgrades();

	tokio::pin!(conn);

	loop {
		tokio::select! {
			result = conn.as_mut() => {
				if let Err(err) = result {
					if !err.is_closed() {
						eprintln!("Connection error: {}", err);
					}
				}
				break;
			}
			_ = shutdown_rx.changed() => {
				conn.as_mut().graceful_shutdown();
			}
		}
	}
}

/// Constrói o service hyper que despacha para o router.
fn make_service(
	router: Arc<MurRouter>,
	rate_limit: usize,
) -> impl Service<
	Request<Incoming>,
	Response = Response<Full<Bytes>>,
	Error = MurError,
	Future = impl Future<Output = Result<Response<Full<Bytes>>, MurError>>,
> {
	service_fn(move |req| {
		let router = Arc::clone(&router);
		async move {
			let req = limited_body_extraction(req, rate_limit).await;
			match router.handle_direct(req).await {
				Ok(res) => Ok(res),
				Err(err) => Ok(err.into_response()),
			}
		}
	})
}

/// Aguarda até todas as conexões ativas fecharem ou o timeout estourar.
async fn wait_for_connections(active: &AtomicUsize, timeout: Duration) {
	println!(
		"Waiting for {} active connection(s) to finish...",
		active.load(Ordering::Relaxed)
	);

	let start = Instant::now();
	while active.load(Ordering::Relaxed) > 0 {
		if start.elapsed() > timeout {
			println!("Shutdown timeout exceeded, forcing shutdown...");
			break;
		}
		tokio::time::sleep(Duration::from_millis(100)).await;
	}
}

/// Sinal de shutdown padrão: Ctrl+C.
fn default_shutdown_signal() -> impl Future<Output = ()> + Send + 'static {
	async {
		tokio::signal::ctrl_c()
			.await
			.expect("Failed to listen for Ctrl+C");
	}
}
