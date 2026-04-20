use murgamu::prelude::*;

#[injectable]
pub struct AppService {}

impl AppService {
	pub fn new() -> Self {
		Self {}
	}

	pub fn is_alive(&self) -> &str {
		"is alive!"
	}
}

