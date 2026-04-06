use crate::mods::token::models::JwtPayload;
use murgamu::prelude::*;

#[injectable]
pub struct TokenService;

impl TokenService {
	fn new() -> Self {
		Self {}
	}

	// TODO: service stopped to require self param
	pub async fn validate_token(&self, token: &str) -> Result<JwtPayload, MurError> {
		if self.is_recorded(token).await {
			return JwtPayload::decode(token);
		}
		Err(MurError::forbidden("Token was not found"))
	}

	async fn is_recorded(&self, _token: &str) -> bool {
		true
	}
}
