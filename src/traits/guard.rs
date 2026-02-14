use crate::mur_http::request::MurRequestContext;
use crate::types::MurRes;
use std::future::Future;
use std::pin::Pin;

pub trait MurGuard: Send + Sync + 'static {
	fn can_activate<'a>(&'a self, ctx: &'a MurRequestContext) -> MurGuardFuture<'a>;

	fn rejection_response(&self) -> MurRes {
		crate::types::MurHttpResponse::forbidden().json(serde_json::json!({
			"error": "Forbidden",
			"message": "Access denied by guard"
		}))
	}

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}

pub type MurGuardFuture<'a> = Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

pub trait MurGuardSync: Send + Sync + 'static {
	fn can_activate_sync(&self, ctx: &MurRequestContext) -> bool;
}
