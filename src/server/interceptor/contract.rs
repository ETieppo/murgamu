use crate::server::aliases::MurRes;
use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use crate::{MurInjects, MurServiceContainer};
use std::future::Future;
use std::pin::Pin;

pub trait MurInterceptorFactory: MurInterceptor {
	fn __create_factory(injects: &MurInjects, container: &MurServiceContainer) -> Self
	where
		Self: Sized;
}

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
