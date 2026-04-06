use crate::server::error::MurError;
use crate::server::http::MurRequestContext;
use crate::server::interceptor::MurInterceptor;
use crate::server::aliases::MurRes;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

type BeforeFnType = Arc<dyn Fn(&MurRequestContext) -> Result<(), MurError> + Send + Sync>;
type AfterFnType = Arc<dyn Fn(&MurRequestContext, MurRes) -> MurRes + Send + Sync>;

pub struct MockInterceptor {
	pub name: String,
	pub before_fn: Option<BeforeFnType>,
	pub after_fn: Option<AfterFnType>,
	pub call_count: Arc<AtomicUsize>,
}

impl MockInterceptor {
	pub fn new() -> Self {
		Self {
			name: "MockInterceptor".to_string(),
			before_fn: None,
			after_fn: None,
			call_count: Arc::new(AtomicUsize::new(0)),
		}
	}

	pub fn named(name: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			..Self::new()
		}
	}

	pub fn before<F>(mut self, f: F) -> Self
	where
		F: Fn(&MurRequestContext) -> Result<(), MurError> + Send + Sync + 'static,
	{
		self.before_fn = Some(Arc::new(f));
		self
	}

	pub fn after<F>(mut self, f: F) -> Self
	where
		F: Fn(&MurRequestContext, MurRes) -> MurRes + Send + Sync + 'static,
	{
		self.after_fn = Some(Arc::new(f));
		self
	}

	pub fn call_count(&self) -> usize {
		self.call_count.load(Ordering::SeqCst)
	}

	pub fn reset(&self) {
		self.call_count.store(0, Ordering::SeqCst);
	}
}

impl Default for MockInterceptor {
	fn default() -> Self {
		Self::new()
	}
}

impl Clone for MockInterceptor {
	fn clone(&self) -> Self {
		Self {
			name: self.name.clone(),
			before_fn: self.before_fn.clone(),
			after_fn: self.after_fn.clone(),
			call_count: Arc::clone(&self.call_count),
		}
	}
}

impl MurInterceptor for MockInterceptor {
	fn before(
		&self,
		ctx: &MurRequestContext,
	) -> Pin<Box<dyn Future<Output = Result<(), MurError>> + Send>> {
		self.call_count.fetch_add(1, Ordering::SeqCst);

		if let Some(ref before_fn) = self.before_fn {
			let result = before_fn(ctx);
			Box::pin(async move { result })
		} else {
			Box::pin(async { Ok(()) })
		}
	}

	fn after(
		&self,
		ctx: &MurRequestContext,
		response: MurRes,
	) -> Pin<Box<dyn Future<Output = MurRes> + Send>> {
		if let Some(ref after_fn) = self.after_fn {
			let result = after_fn(ctx, response);
			Box::pin(async move { result })
		} else {
			Box::pin(async move { response })
		}
	}

	fn name(&self) -> &str {
		&self.name
	}
}
