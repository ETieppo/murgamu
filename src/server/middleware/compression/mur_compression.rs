use super::algorithm::MurCompressionAlgorithm;
use super::config::MurCompressionConfig;
use super::deflate::MurDeflateEncoder;
use super::gzip::MurGzipEncoder;
use super::level::MurCompressionLevel;
use crate::server::aliases::MurFuture;
use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use crate::server::middleware::{MurMiddleware, MurNext};
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::{HeaderValue, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use hyper::Response;
use std::sync::Arc;

#[derive(Clone)]
pub struct MurCompression {
	config: Arc<MurCompressionConfig>,
}

impl MurCompression {
	pub fn new() -> Self {
		Self {
			config: Arc::new(MurCompressionConfig::default()),
		}
	}

	pub fn gzip_only() -> Self {
		Self {
			config: Arc::new(MurCompressionConfig {
				algorithms: vec![MurCompressionAlgorithm::Gzip],
				..Default::default()
			}),
		}
	}

	pub fn brotli_only() -> Self {
		Self {
			config: Arc::new(MurCompressionConfig {
				algorithms: vec![MurCompressionAlgorithm::Brotli],
				..Default::default()
			}),
		}
	}

	pub fn from_config(config: MurCompressionConfig) -> Self {
		Self {
			config: Arc::new(config),
		}
	}

	pub fn gzip(mut self) -> Self {
		let mut config = (*self.config).clone();
		if !config.algorithms.contains(&MurCompressionAlgorithm::Gzip) {
			config.algorithms.push(MurCompressionAlgorithm::Gzip);
		}
		self.config = Arc::new(config);
		self
	}

	pub fn brotli(mut self) -> Self {
		let mut config = (*self.config).clone();
		if !config.algorithms.contains(&MurCompressionAlgorithm::Brotli) {
			config.algorithms.push(MurCompressionAlgorithm::Brotli);
		}
		self.config = Arc::new(config);
		self
	}

	pub fn deflate(mut self) -> Self {
		let mut config = (*self.config).clone();
		if !config
			.algorithms
			.contains(&MurCompressionAlgorithm::Deflate)
		{
			config.algorithms.push(MurCompressionAlgorithm::Deflate);
		}
		self.config = Arc::new(config);
		self
	}

	pub fn level(mut self, level: MurCompressionLevel) -> Self {
		let mut config = (*self.config).clone();
		config.level = level;
		self.config = Arc::new(config);
		self
	}

	pub fn min_size(mut self, size: usize) -> Self {
		let mut config = (*self.config).clone();
		config.min_size = size;
		self.config = Arc::new(config);
		self
	}

	pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
		let mut config = (*self.config).clone();
		config.content_types.push(content_type.into());
		self.config = Arc::new(config);
		self
	}

	pub fn exclude_type(mut self, content_type: impl Into<String>) -> Self {
		let mut config = (*self.config).clone();
		config.exclude_types.push(content_type.into());
		self.config = Arc::new(config);
		self
	}

	pub fn compress_without_accept_encoding(mut self, enable: bool) -> Self {
		let mut config = (*self.config).clone();
		config.compress_without_accept_encoding = enable;
		self.config = Arc::new(config);
		self
	}

	pub fn should_compress_content_type(&self, content_type: Option<&str>) -> bool {
		let content_type = match content_type {
			Some(ct) => ct.to_lowercase(),
			None => return false,
		};

		for excluded in &self.config.exclude_types {
			if content_type.starts_with(excluded) || content_type.contains(excluded) {
				return false;
			}
		}

		if !self.config.content_types.is_empty() {
			return self
				.config
				.content_types
				.iter()
				.any(|ct| content_type.starts_with(ct) || content_type.contains(ct));
		}

		content_type.starts_with("text/")
			|| content_type.contains("json")
			|| content_type.contains("xml")
			|| content_type.contains("javascript")
			|| content_type.contains("css")
			|| content_type.contains("html")
			|| content_type.contains("svg")
	}

	fn select_algorithm(&self, accept_encoding: Option<&str>) -> Option<MurCompressionAlgorithm> {
		let accept_encoding = match accept_encoding {
			Some(ae) => ae,
			None => {
				if self.config.compress_without_accept_encoding {
					return self.config.algorithms.first().copied();
				}
				return None;
			}
		};

		let client_prefs = MurCompressionAlgorithm::from_accept_encoding(accept_encoding);

		for (algo, quality) in client_prefs {
			if quality > 0.0 && self.config.algorithms.contains(&algo) {
				return Some(algo);
			}
		}

		None
	}

	fn compress(&self, data: &[u8], algorithm: MurCompressionAlgorithm) -> Option<Vec<u8>> {
		match algorithm {
			MurCompressionAlgorithm::Gzip => self.compress_gzip(data),
			MurCompressionAlgorithm::Deflate => self.compress_deflate(data),
			MurCompressionAlgorithm::Brotli => self.compress_brotli(data),
			MurCompressionAlgorithm::Identity => Some(data.to_vec()),
		}
	}

	fn compress_gzip(&self, data: &[u8]) -> Option<Vec<u8>> {
		let encoder = MurGzipEncoder::new(self.config.level.gzip_level());
		encoder.compress(data)
	}

	fn compress_deflate(&self, data: &[u8]) -> Option<Vec<u8>> {
		let encoder = MurDeflateEncoder::new(self.config.level.gzip_level());
		encoder.compress(data)
	}

	fn compress_brotli(&self, data: &[u8]) -> Option<Vec<u8>> {
		self.compress_gzip(data)
	}
}

impl Default for MurCompression {
	fn default() -> Self {
		Self::new()
	}
}

impl std::fmt::Debug for MurCompression {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurCompression")
			.field("config", &self.config)
			.finish()
	}
}

impl MurMiddleware for MurCompression {
	fn handle(&self, ctx: MurRequestContext, next: MurNext) -> MurFuture {
		let compression = self.clone();

		Box::pin(async move {
			let accept_encoding = ctx.header(ACCEPT_ENCODING.as_str()).map(|s| s.to_string());
			let result = next.run(ctx).await;

			match result {
				Ok(response) => {
					if response.headers().contains_key(CONTENT_ENCODING) {
						return Ok(response);
					}

					let content_type = response
						.headers()
						.get(CONTENT_TYPE)
						.and_then(|v| v.to_str().ok())
						.map(|s| s.to_string());

					if !compression.should_compress_content_type(content_type.as_deref()) {
						return Ok(response);
					}

					let (parts, body) = response.into_parts();
					let collected = match body.collect().await {
						Ok(c) => c.to_bytes(),
						Err(_) => {
							return Err(MurError::Internal("Failed to read response body".into()))
						}
					};

					if collected.len() < compression.config.min_size {
						let response = Response::from_parts(parts, Full::new(collected));
						return Ok(response);
					}

					let algorithm = match compression.select_algorithm(accept_encoding.as_deref()) {
						Some(algo) => algo,
						None => {
							let response = Response::from_parts(parts, Full::new(collected));
							return Ok(response);
						}
					};

					match compression.compress(&collected, algorithm) {
						Some(compressed) => {
							if compressed.len() >= collected.len() {
								let response = Response::from_parts(parts, Full::new(collected));
								return Ok(response);
							}

							let mut response = Response::from_parts(
								parts,
								Full::new(Bytes::from(compressed.clone())),
							);

							response.headers_mut().insert(
								CONTENT_ENCODING,
								HeaderValue::from_static(algorithm.as_str()),
							);

							response.headers_mut().insert(
								CONTENT_LENGTH,
								HeaderValue::from_str(&compressed.len().to_string()).unwrap(),
							);

							if let Ok(vary) = HeaderValue::from_str("Accept-Encoding") {
								response.headers_mut().insert("vary", vary);
							}

							Ok(response)
						}
						None => {
							let response = Response::from_parts(parts, Full::new(collected));
							Ok(response)
						}
					}
				}
				Err(e) => Err(e),
			}
		})
	}

	fn name(&self) -> &str {
		"MurCompression"
	}
}
