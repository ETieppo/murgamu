use crate::server::error::MurError;
use crate::server::http::request::MurRequestContext;
use std::future::Future;
use std::pin::Pin;

/// Asynchronous extractor trait for request data.
///
/// Implement `MurExtractor` to create a custom type that can be extracted from a
/// [`MurRequestContext`] asynchronously. The framework calls [`extract`](MurExtractor::extract)
/// automatically when the type appears as a handler parameter.
///
/// For synchronous extraction prefer [`MurExtractorSync`].
pub trait MurExtractor: Sized + Send {
	/// The error type returned when extraction fails.
	///
	/// Must be convertible to [`MurError`] so the framework can produce an
	/// appropriate HTTP error response.
	type Error: Into<MurError>;

	/// Attempts to extract `Self` from the incoming request context.
	fn extract(
		ctx: &MurRequestContext,
	) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + '_>>;
}

/// Synchronous extractor trait for request data.
///
/// Implement `MurExtractorSync` when extraction does not require `async` I/O.
/// This is the preferred interface for lightweight extractors (headers, path
/// params, query strings, etc.).
pub trait MurExtractorSync: Sized + Send {
	/// The error type returned when extraction fails.
	///
	/// Must be convertible to [`MurError`].
	type Error: Into<MurError>;

	/// Synchronously extracts `Self` from the incoming request context.
	fn extract_sync(ctx: &MurRequestContext) -> Result<Self, Self::Error>;
}
