use super::config::MurTlsConfig;
use super::error::MurTlsError;
use rustls::ServerConfig;
use std::sync::Arc;
use tokio_rustls::TlsAcceptor;

#[derive(Clone)]
pub struct MurTlsAcceptor {
	pub inner: TlsAcceptor,
}

impl MurTlsAcceptor {
	pub fn new(config: &MurTlsConfig) -> Result<Self, MurTlsError> {
		Ok(Self {
			inner: config.build_acceptor()?,
		})
	}

	pub fn from_config(config: ServerConfig) -> Self {
		Self {
			inner: TlsAcceptor::from(Arc::new(config)),
		}
	}

	pub fn inner(&self) -> &TlsAcceptor {
		&self.inner
	}

	pub async fn accept<S>(
		&self,
		stream: S,
	) -> Result<tokio_rustls::server::TlsStream<S>, std::io::Error>
	where
		S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
	{
		self.inner.accept(stream).await
	}
}

impl std::fmt::Debug for MurTlsAcceptor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurTlsAcceptor").finish()
	}
}
