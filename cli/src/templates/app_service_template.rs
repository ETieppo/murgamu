pub const SERVICE_TEMPLATE: &str = r#"use murgamu::prelude::*;
use crate::mods::app::models::user_props::UserProps;

#[service]
pub struct AppService;

impl AppService {
  pub fn new() -> Self {
    Self
  }

  pub fn greet(&self, name: String) -> String {
    format!("Hello, {}! Welcome to Murgamü.", name)
  }

  pub fn get_user(&self, id: u32) -> UserProps {
    UserProps {
      id,
      username: String::from("Murgamü"),
    }
  }
}
"#;
