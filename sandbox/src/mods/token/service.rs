use std::hash::RandomState;

use murgamu::prelude::*;

#[service]
pub struct TokenService;

impl TokenService {
	fn new() -> Self {
		Self {}
	}

	// TODO: service stopped to require self param
	pub async fn validate_token(&self, cookies: HashMap<String, String, RandomState>) -> bool {
		cookies.contains_key("authorization")
	}
}
