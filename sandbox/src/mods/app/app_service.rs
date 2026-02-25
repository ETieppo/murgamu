use super::models::user_props::UserProps;
use crate::InjectNameService;
use murgamu::prelude::*;

#[service]
pub struct AppService {
	injected_name_service: Arc<InjectNameService>,
}

impl AppService {
	pub fn new(injected_name_service: Arc<InjectNameService>) -> Self {
		Self {
			injected_name_service,
		}
	}

	pub fn greet(&self) -> Result<String, MurError> {
		let name = self.injected_name_service.get_name()?;
		Ok(format!("Hello, {}! Welcome to Murgamü.", name))
	}

	pub fn get_user(&self, id: u32) -> UserProps {
		if id == 1 {
			UserProps {
				id,
				username: String::from("Murgamü"),
			}
		} else {
			UserProps {
				id,
				username: self
					.injected_name_service
					.get_name()
					.expect("Error getting project name"),
			}
		}
	}
}
