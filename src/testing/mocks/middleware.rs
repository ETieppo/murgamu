use crate::mur_http::request::MurRequestContext;
use crate::traits::{MurMiddleware, MurNext};
use crate::types::MurFuture;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct MockMiddleware {
	pub name: String,
	pub handler: Option<Arc<dyn Fn(MurRequestContext, MurNext) -> MurFuture + Send + Sync>>,
	pub call_count: Arc<AtomicUsize>,
	pub pass_through: bool,
}

impl MockMiddleware {
	pub fn new() -> Self {
		Self {
			name: "MockMiddleware".to_string(),
			handler: None,
			call_count: Arc::new(AtomicUsize::new(0)),
			pass_through: true,
		}
	}

	pub fn named(name: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			..Self::new()
		}
	}

	pub fn with_handler<F>(mut self, handler: F) -> Self
	where
		F: Fn(MurRequestContext, MurNext) -> MurFuture + Send + Sync + 'static,
	{
		self.handler = Some(Arc::new(handler));
		self.pass_through = false;
		self
	}

	pub fn blocking(mut self) -> Self {
		self.pass_through = false;
		self.handler = Some(Arc::new(|_ctx, _next| {
			Box::pin(async {
				crate::types::MurHttpResponse::forbidden().json(serde_json::json!({
					"error": "Blocked by mock middleware"
				}))
			})
		}));
		self
	}

	pub fn add_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
		let header_name = name.into();
		let header_value = value.into();

		self.handler = Some(Arc::new(move |ctx, next| {
			let name = header_name.clone();
			let value = header_value.clone();
			Box::pin(async move {
				let response = next.run(ctx).await;
				match response {
					Ok(mut res) => {
						res.headers_mut().insert(
							name.parse::<http::header::HeaderName>().unwrap(),
							value.parse::<http::header::HeaderValue>().unwrap(),
						);
						Ok(res)
					}
					Err(e) => Err(e),
				}
			})
		}));
		self
	}

	pub fn call_count(&self) -> usize {
		self.call_count.load(Ordering::SeqCst)
	}

	pub fn reset(&self) {
		self.call_count.store(0, Ordering::SeqCst);
	}
}

impl Default for MockMiddleware {
	fn default() -> Self {
		Self::new()
	}
}

impl Clone for MockMiddleware {
	fn clone(&self) -> Self {
		Self {
			name: self.name.clone(),
			handler: self.handler.clone(),
			call_count: Arc::clone(&self.call_count),
			pass_through: self.pass_through,
		}
	}
}

impl MurMiddleware for MockMiddleware {
	fn handle(&self, ctx: MurRequestContext, next: MurNext) -> MurFuture {
		self.call_count.fetch_add(1, Ordering::SeqCst);

		if let Some(ref handler) = self.handler {
			handler(ctx, next)
		} else if self.pass_through {
			next.run(ctx)
		} else {
			Box::pin(async {
				crate::types::MurHttpResponse::internal_error().json(serde_json::json!({
					"error": "Mock middleware not configured"
				}))
			})
		}
	}

	fn name(&self) -> &str {
		&self.name
	}
}
