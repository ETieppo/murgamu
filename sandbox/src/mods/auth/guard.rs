use crate::mods::token::TokenService;
use murgamu::prelude::*;

#[guard]
pub struct GlobalGuard {
	token_service: Arc<TokenService>,
}

impl GlobalGuard {
	pub fn new(token_service: Arc<TokenService>) -> Self {
		Self { token_service }
	}

	pub async fn can_activate(&self, ctx: &MurRequestContext) -> bool {
		self.token_service.validate_token(ctx.cookies()).await
	}
}
