use crate::MurInjects;
use crate::MurServiceContainer;
use crate::server::aliases::MurRes;
use crate::server::http::MurHttpResponse;
use crate::server::http::MurRequestContext;
use std::any::Any;
use std::future::Future;
use std::pin::Pin;

/// Pinned boxed future returned by [`MurGuard::check_can_activate`].
pub type MurGuardFuture<'a> = Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

/// Asynchronous route guard.
///
/// Implement `MurGuard` to protect routes from unauthorized access. Guards are
/// evaluated before the route handler is invoked. If any guard returns `false`
/// the request is rejected with [`rejection_response`](MurGuard::rejection_response).
///
/// Guards can be registered globally on the server (applying to every route) or
/// per-route via the `#[guard]` attribute. DI-enabled guards should implement
/// [`MurGuardFactory`] as well.
///
/// # Example
///
/// ```rust,ignore
/// #[guard]
/// struct AuthGuard {
///     secret: String,
/// }
///
/// impl MurGuard for AuthGuard {
///     fn check_can_activate<'a>(&'a self, ctx: &'a MurRequestContext) -> MurGuardFuture<'a> {
///         Box::pin(async move {
///             ctx.header("Authorization")
///                 .map(|h| h == self.secret.as_str())
///                 .unwrap_or(false)
///         })
///     }
/// }
/// ```
pub trait MurGuard: Send + Sync + 'static {
	/// Determines whether the request is allowed to proceed.
	///
	/// Return `true` to allow the request or `false` to reject it.
	fn check_can_activate<'a>(&'a self, ctx: &'a MurRequestContext) -> MurGuardFuture<'a>;

	/// The response sent to the client when this guard rejects a request.
	///
	/// Defaults to `403 Forbidden` with a JSON error body.
	fn rejection_response(&self) -> MurRes {
		MurHttpResponse::forbidden().json(serde_json::json!({
			"error": "Forbidden",
			"message": "Access denied!"
		}))
	}

	/// Returns the guard's human-readable name (defaults to the type name).
	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}

	/// Allows downcasting to the concrete guard type.
	fn as_any(&self) -> &dyn Any;
}

/// Dependency-injection factory for [`MurGuard`] implementations.
///
/// The framework calls `__create_factory` when building the guard instance from
/// the DI container. This trait is generated automatically by the `#[guard]`
/// derive macro when the struct has injected fields.
pub trait MurGuardFactory: MurGuard {
	/// Constructs the guard using dependencies from `injects` and `container`.
	fn __create_factory(injects: &MurInjects, _container: &MurServiceContainer) -> Self;
}

/// Synchronous variant of [`MurGuard`] for guards that do not need async I/O.
pub trait MurGuardSync: Send + Sync + 'static {
	/// Synchronously determines whether the request is allowed to proceed.
	fn can_activate_sync(&self, ctx: &MurRequestContext) -> bool;
}
