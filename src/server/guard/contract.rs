use crate::server::aliases::MurRes;
use crate::server::http::MurHttpResponse;
use crate::server::http::MurRequestContext;
use std::future::Future;
use std::pin::Pin;

pub type MurGuardFuture<'a> = Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

pub trait MurGuard: Send + Sync + 'static {
	fn can_activate<'a>(&'a self, ctx: &'a MurRequestContext) -> MurGuardFuture<'a>;

	fn rejection_response(&self) -> MurRes {
		MurHttpResponse::forbidden().json(serde_json::json!({
			"error": "Forbidden",
			"message": "Access denied by guard"
		}))
	}

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}

pub trait MurGuardSync: Send + Sync + 'static {
	fn can_activate_sync(&self, ctx: &MurRequestContext) -> bool;
}
