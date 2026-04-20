use super::MurError;
use crate::server::aliases::MurRes;
use crate::server::http::MurRequestContext;

/// Trait for handling specific error types and converting them into HTTP responses.
///
/// Implement `MurExceptionFilter` to intercept [`MurError`] values before
/// they reach the default error handler. Multiple filters can be registered
/// on the server; each filter is consulted in registration order and the first
/// one whose [`can_handle`](MurExceptionFilter::can_handle) returns `true`
/// processes the error.
///
/// # Example
///
/// ```rust,ignore
/// struct NotFoundFilter;
///
/// impl MurExceptionFilter for NotFoundFilter {
///     fn can_handle(&self, error: &MurError) -> bool {
///         matches!(error, MurError::NotFound(_))
///     }
///
///     fn catch(&self, error: MurError, _ctx: &MurRequestContext) -> MurRes {
///         MurHttpResponse::not_found().json(serde_json::json!({
///             "message": error.message(),
///             "code": "RESOURCE_NOT_FOUND",
///         }))
///     }
/// }
/// ```
pub trait MurExceptionFilter: Send + Sync + 'static {
	/// Converts `error` into an HTTP response.
	///
	/// Called only when [`can_handle`](MurExceptionFilter::can_handle) returns `true`.
	fn catch(&self, error: MurError, ctx: &MurRequestContext) -> MurRes;

	/// Returns `true` if this filter should handle `error`.
	///
	/// The default implementation accepts every error, making this filter a
	/// catch-all. Override to restrict handling to specific variants.
	fn can_handle(&self, _error: &MurError) -> bool {
		true
	}

	/// Returns the filter's human-readable name (defaults to the type name).
	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}
