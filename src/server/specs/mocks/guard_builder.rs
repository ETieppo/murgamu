use super::MockGuard;
use crate::server::http::MurRequestContext;
use std::sync::atomic::Ordering;
use std::sync::Arc;

type CheckFnType = Arc<dyn Fn(&MurRequestContext) -> bool + Send + Sync>;

pub struct MockGuardBuilder {
	name: String,
	allow: bool,
	check_fn: Option<CheckFnType>,
}

impl MockGuardBuilder {
	pub fn new() -> Self {
		Self {
			name: "MockGuard".to_string(),
			allow: true,
			check_fn: None,
		}
	}

	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn allow_all(mut self) -> Self {
		self.allow = true;
		self.check_fn = None;
		self
	}

	pub fn deny_all(mut self) -> Self {
		self.allow = false;
		self.check_fn = None;
		self
	}

	pub fn require_header(mut self, header: impl Into<String>) -> Self {
		let header = header.into();
		self.check_fn = Some(Arc::new(move |ctx| ctx.header(&header).is_some()));
		self
	}

	pub fn require_header_value(
		mut self,
		header: impl Into<String>,
		value: impl Into<String>,
	) -> Self {
		let header = header.into();
		let value = value.into();
		self.check_fn = Some(Arc::new(move |ctx| {
			ctx.header(&header).map(|h| h == value).unwrap_or(false)
		}));
		self
	}

	pub fn with_check<F>(mut self, check: F) -> Self
	where
		F: Fn(&MurRequestContext) -> bool + Send + Sync + 'static,
	{
		self.check_fn = Some(Arc::new(check));
		self
	}

	pub fn build(self) -> super::MockGuard {
		let guard = MockGuard::named(self.name);
		guard.allow.store(self.allow, Ordering::SeqCst);
		MockGuard {
			check_fn: self.check_fn,
			..guard
		}
	}
}

impl Default for MockGuardBuilder {
	fn default() -> Self {
		Self::new()
	}
}
