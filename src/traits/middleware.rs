use crate::mur_http::request::MurRequestContext;
use crate::types::MurFuture;
use std::sync::Arc;

pub trait MurMiddleware: Send + Sync + 'static {
	fn handle(&self, ctx: MurRequestContext, next: MurNext) -> MurFuture;

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}

pub struct MurNext {
	pub(crate) handler: Arc<dyn Fn(MurRequestContext) -> MurFuture + Send + Sync>,
}

impl MurNext {
	pub fn new(handler: Arc<dyn Fn(MurRequestContext) -> MurFuture + Send + Sync>) -> Self {
		Self { handler }
	}

	pub fn run(self, ctx: MurRequestContext) -> MurFuture {
		(self.handler)(ctx)
	}
}
