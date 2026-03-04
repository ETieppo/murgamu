use super::MockGuard;
use crate::server::http::MurRequestContext;
use std::sync::Arc;
use std::sync::atomic::Ordering;

type CanActivateFnType = Arc<dyn Fn(&MurRequestContext) -> bool + Send + Sync>;

pub struct MockGuardBuilder {
	name: String,
	allow: bool,
	can_activate_fn: Option<CanActivateFnType>,
}

impl MockGuardBuilder {
	pub fn new() -> Self {
		Self {
			name: "MockGuard".to_string(),
			allow: true,
			can_activate_fn: None,
		}
	}

	pub fn name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn allow_all(mut self) -> Self {
		self.allow = true;
		self.can_activate_fn = None;
		self
	}

	pub fn deny_all(mut self) -> Self {
		self.allow = false;
		self.can_activate_fn = None;
		self
	}

	pub fn require_header(mut self, header: impl Into<String>) -> Self {
		let header = header.into();
		self.can_activate_fn =
			Some(Arc::new(move |ctx| ctx.header(&header).is_some()));
		self
	}

	pub fn require_header_value(
		mut self,
		header: impl Into<String>,
		value: impl Into<String>,
	) -> Self {
		let header = header.into();
		let value = value.into();
		self.can_activate_fn = Some(Arc::new(move |ctx| {
			ctx.header(&header).map(|h| h == value).unwrap_or(false)
		}));
		self
	}

	pub fn with_check<F>(mut self, check: F) -> Self
	where
		F: Fn(&MurRequestContext) -> bool + Send + Sync + 'static,
	{
		self.can_activate_fn = Some(Arc::new(check));
		self
	}

	pub fn build(self) -> super::MockGuard {
		let guard = MockGuard::named(self.name);
		guard.allow.store(self.allow, Ordering::SeqCst);
		MockGuard {
			can_activate_fn: self.can_activate_fn,
			..guard
		}
	}
}

impl Default for MockGuardBuilder {
	fn default() -> Self {
		Self::new()
	}
}
