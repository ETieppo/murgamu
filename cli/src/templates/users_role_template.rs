pub const USERS_ROLE_TEMPLATE: &str = r#"use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, strum_macros::Display)]
pub enum UserRole {
	Common,
}
"#;
