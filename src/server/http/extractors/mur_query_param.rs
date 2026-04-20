use crate::server::http::MurRequestContext;
use std::marker::PhantomData;

/// A wrapper for a single optional URL query parameter.
///
/// `MurQueryParam<T>` extracts one named query string key and attempts to
/// parse it into `T`. When the key is absent or unparseable the inner value
/// is `None`, making this extractor infallible at the handler boundary.
///
/// # Usage in route handlers
///
/// ```rust,ignore
/// #[get("/items")]
/// async fn list_items(&self, ctx: MurRequestContext) -> MurRes {
///     let page = MurQueryParam::<u32>::extract(&ctx, "page");
///     let page_num = page.unwrap_or(1);
///     mur_json!(serde_json::json!({ "page": page_num }))
/// }
/// ```
///
/// For deserializing the full query string into a struct, use [`MurQuery<T>`](crate::MurQuery).
#[derive(Debug, Clone)]
pub struct MurQueryParam<T> {
	/// The parsed value, or `None` if the parameter was absent or invalid.
	pub value: Option<T>,
	_marker: PhantomData<T>,
}

impl<T> MurQueryParam<T> {
	/// Creates a `MurQueryParam` wrapping `value`.
	pub fn new(value: Option<T>) -> Self {
		Self {
			value,
			_marker: PhantomData,
		}
	}

	/// Returns `true` if the parameter was present and successfully parsed.
	pub fn is_present(&self) -> bool {
		self.value.is_some()
	}

	/// Returns the inner value, or `default` if the parameter was absent.
	pub fn unwrap_or(self, default: T) -> T {
		self.value.unwrap_or(default)
	}

	/// Returns the inner value, or computes it from `f` if the parameter was absent.
	pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
		self.value.unwrap_or_else(f)
	}
}

impl<T: std::str::FromStr> MurQueryParam<T> {
	/// Extracts the named query parameter from the request and attempts to
	/// parse it into `T`.
	///
	/// Silently returns `None` when the key is missing or parsing fails —
	/// this extractor is always successful.
	pub fn extract(ctx: &MurRequestContext, name: &str) -> Self {
		let value = ctx.query_param(name).and_then(|s| s.parse().ok());
		Self {
			value,
			_marker: PhantomData,
		}
	}
}
