use serde::Serialize;

#[derive(Serialize)]
pub struct UserProps {
	pub id: u32,
	pub username: String,
}
