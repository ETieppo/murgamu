use crate::core::error::MurError;
use crate::mur_http::request::MurRequestContext;
use crate::types::MurRes;
use std::future::Future;
use std::pin::Pin;

pub trait MurInterceptor: Send + Sync + 'static {
	fn before(&self, _ctx: &MurRequestContext) -> MurInterceptorFuture {
		Box::pin(async { Ok(()) })
	}

	fn after(
		&self,
		_ctx: &MurRequestContext,
		response: MurRes,
	) -> Pin<Box<dyn Future<Output = MurRes> + Send>> {
		Box::pin(async move { response })
	}

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}

pub type MurInterceptorFuture = Pin<Box<dyn Future<Output = Result<(), MurError>> + Send>>;
