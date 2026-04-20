use crate::server::aliases::MurRes;
use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use crate::{MurInjects, MurServiceContainer};
use std::future::Future;
use std::pin::Pin;

/// Dependency-injection factory for [`MurInterceptor`] implementations.
///
/// Generated automatically by the `#[interceptor]` derive macro when the
/// struct has injected fields.
pub trait MurInterceptorFactory: MurInterceptor {
	/// Constructs the interceptor using dependencies from `injects` and `container`.
	fn __create_factory(injects: &MurInjects, container: &MurServiceContainer) -> Self
	where
		Self: Sized;
}

/// Middleware-like hook that wraps individual route handler invocations.
///
/// `MurInterceptor` is called immediately before and after every matched
/// route handler. Use it to implement cross-cutting concerns such as logging,
/// metrics, response transformation, or rate-limiting at the handler level.
///
/// # Lifecycle
///
/// 1. [`before`](MurInterceptor::before) — runs before the handler. Returning
///    `Err` short-circuits execution and propagates the error.
/// 2. The route handler executes.
/// 3. [`after`](MurInterceptor::after) — receives the handler's response and
///    can transform it before it is sent to the client.
///
/// # Example
///
/// ```rust,ignore
/// #[interceptor]
/// struct LoggingInterceptor;
///
/// impl MurInterceptor for LoggingInterceptor {
///     fn before(&self, ctx: &MurRequestContext) -> MurInterceptorFuture {
///         let path = ctx.parts.uri.path().to_string();
///         Box::pin(async move {
///             println!("→ {}", path);
///             Ok(())
///         })
///     }
/// }
/// ```
pub trait MurInterceptor: Send + Sync + 'static {
	/// Called before the route handler.
	///
	/// Returning `Err` aborts execution and sends the error as the response.
	/// The default implementation is a no-op.
	fn before(&self, _ctx: &MurRequestContext) -> MurInterceptorFuture {
		Box::pin(async { Ok(()) })
	}

	/// Called after the route handler with its response.
	///
	/// The interceptor may transform the response before it is returned to
	/// the client. The default implementation passes the response unchanged.
	fn after(
		&self,
		_ctx: &MurRequestContext,
		response: MurRes,
	) -> Pin<Box<dyn Future<Output = MurRes> + Send>> {
		Box::pin(async move { response })
	}

	/// Returns the interceptor's human-readable name (defaults to the type name).
	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}

/// Pinned boxed future returned by [`MurInterceptor::before`].
pub type MurInterceptorFuture = Pin<Box<dyn Future<Output = Result<(), MurError>> + Send>>;
