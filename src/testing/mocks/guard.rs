use crate::mur_http::request::MurRequestContext;
use crate::traits::{MurGuard, MurGuardFuture};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub type GuardCheckFn = Arc<dyn Fn(&MurRequestContext) -> bool + Send + Sync>;

pub struct MockGuard {
	pub name: String,
	pub allow: Arc<AtomicBool>,
	pub check_fn: Option<GuardCheckFn>,
	pub call_count: Arc<AtomicUsize>,
}

impl MockGuard {
	pub fn new() -> Self {
		Self {
			name: "MockGuard".to_string(),
			allow: Arc::new(AtomicBool::new(true)),
			check_fn: None,
			call_count: Arc::new(AtomicUsize::new(0)),
		}
	}

	pub fn named(name: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			..Self::new()
		}
	}

	pub fn allow_all(self) -> Self {
		self.allow.store(true, Ordering::SeqCst);
		Self {
			check_fn: None,
			..self
		}
	}

	pub fn deny_all(self) -> Self {
		self.allow.store(false, Ordering::SeqCst);
		Self {
			check_fn: None,
			..self
		}
	}

	pub fn with_check<F>(self, check: F) -> Self
	where
		F: Fn(&MurRequestContext) -> bool + Send + Sync + 'static,
	{
		Self {
			check_fn: Some(Arc::new(check)),
			..self
		}
	}

	pub fn call_count(&self) -> usize {
		self.call_count.load(Ordering::SeqCst)
	}

	pub fn reset(&self) {
		self.call_count.store(0, Ordering::SeqCst);
	}
}

impl Default for MockGuard {
	fn default() -> Self {
		Self::new()
	}
}

impl Clone for MockGuard {
	fn clone(&self) -> Self {
		Self {
			name: self.name.clone(),
			allow: Arc::clone(&self.allow),
			check_fn: self.check_fn.clone(),
			call_count: Arc::clone(&self.call_count),
		}
	}
}

impl MurGuard for MockGuard {
	fn can_activate(&self, ctx: &MurRequestContext) -> MurGuardFuture {
		self.call_count.fetch_add(1, Ordering::SeqCst);

		let result = if let Some(ref check_fn) = self.check_fn {
			check_fn(ctx)
		} else {
			self.allow.load(Ordering::SeqCst)
		};

		Box::pin(async move { result })
	}

	fn name(&self) -> &str {
		&self.name
	}
}
