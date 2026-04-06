use crate::mods::token::TokenService;
use murgamu::prelude::*;

const AUTH_COOKIE: &str = "authorization";

#[guard]
pub struct GlobalGuard {
	token_service: Arc<TokenService>,
}

impl GlobalGuard {
	pub fn new(token_service: Arc<TokenService>) -> Self {
		Self { token_service }
	}

	pub async fn can_activate(&self, ctx: &MurRequestContext) -> bool {
		if ctx.access_control.is_public {
			return true;
		}

		if let Some(token) = ctx.cookie(AUTH_COOKIE)
			&& let Ok(payload) = self.token_service.validate_token(token).await
			&& ctx.has_allowed_role(payload.role)
		{
			return true;
		};

		false
	}
}
