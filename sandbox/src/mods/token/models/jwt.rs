use crate::mods::users::UserRole;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use murgamu::{MurError, server::error::builder::MurResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JwtPayload {
	pub id: usize,
	pub role: UserRole,
	pub lifetime: usize,
}

impl JwtPayload {
	pub fn decode(token: &str) -> MurResult<Self> {
		let token = jsonwebtoken::decode::<JwtPayload>(
			token,
			&DecodingKey::from_secret("secret".as_ref()),
			&Validation::new(jsonwebtoken::Algorithm::EdDSA),
		)?;

		Ok(token.claims)
	}

	pub fn encode(id: usize, role: UserRole) -> Result<String, jsonwebtoken::errors::Error> {
		jsonwebtoken::encode(
			&Header::new(jsonwebtoken::Algorithm::EdDSA),
			&JwtPayload {
				id,
				role,
				lifetime: (chrono::Utc::now().naive_utc() + chrono::naive::Days::new(1))
					.and_utc()
					.timestamp() as usize,
			},
			&EncodingKey::from_secret("secret".as_ref()),
		)
	}
}
