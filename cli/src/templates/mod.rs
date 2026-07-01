pub const CARGO_TEMPLATE: &str = r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"

[dependencies]
chrono = "0.4"
jsonwebtoken = "10"
murgamu = "{{murgamu_version}}"
serde = { version = "1", features = ["derive"] }
strum_macros = "0.28"
"#;

pub const FORMATTER_TEMPLATE: &str = include_str!("../../.rustfmt.toml");
pub const ENV_TEMPLATE: &str = include_str!("../../.env");

pub const FULL_MAIN_TEMPLATE: &str = include_str!("../../template_sources/src/full_main.rs");
pub const FULL_MODULES_MOD_TEMPLATE: &str = include_str!("../../template_sources/src/mods/full_mod.rs");
pub const FULL_APP_MOD_CONTENT_TEMPLATE: &str =
	include_str!("../../template_sources/src/mods/app/full_mod.rs");
pub const FULL_CONTROLLER_TEMPLATE: &str =
	include_str!("../../template_sources/src/mods/app/full_controller.rs");
pub const FULL_SERVICE_TEMPLATE: &str = include_str!("../../template_sources/src/mods/app/full_service.rs");
pub const FULL_MODELS_MOD_TEMPLATE: &str =
	include_str!("../../template_sources/src/mods/app/models/full_mod.rs");
pub const FULL_USER_PROPS_TEMPLATE: &str =
	include_str!("../../template_sources/src/mods/app/models/full_user_props.rs");
pub const FULL_AUTH_MOD_TEMPLATE: &str = include_str!("../../template_sources/src/mods/auth/full_mod.rs");
pub const FULL_AUTH_GUARD_TEMPLATE: &str = include_str!("../../template_sources/src/mods/auth/full_guard.rs");
pub const FULL_AUTH_PIPE_TEMPLATE: &str =
	include_str!("../../template_sources/src/mods/auth/full_jwt_extraction_pipe.rs");
pub const FULL_TOKEN_MOD_TEMPLATE: &str = include_str!("../../template_sources/src/mods/token/full_mod.rs");
pub const FULL_TOKEN_SERVICE_TEMPLATE: &str =
	include_str!("../../template_sources/src/mods/token/full_service.rs");
pub const FULL_TOKEN_MODELS_MOD_TEMPLATE: &str =
	include_str!("../../template_sources/src/mods/token/models/full_mod.rs");
pub const FULL_TOKEN_JWT_TEMPLATE: &str =
	include_str!("../../template_sources/src/mods/token/models/full_jwt.rs");
pub const FULL_USERS_MOD_TEMPLATE: &str = include_str!("../../template_sources/src/mods/users/full_mod.rs");
pub const FULL_USERS_ROLE_TEMPLATE: &str = include_str!("../../template_sources/src/mods/users/full_role.rs");

pub const STARTER_MAIN_TEMPLATE: &str = include_str!("../../template_sources/src/starter_main.rs");
pub const STARTER_MODULES_MOD_TEMPLATE: &str = include_str!("../../template_sources/src/mods/starter_mod.rs");
pub const STARTER_APP_MOD_CONTENT_TEMPLATE: &str =
	include_str!("../../template_sources/src/mods/app/starter_mod.rs");
pub const STARTER_CONTROLLER_TEMPLATE: &str =
	include_str!("../../template_sources/src/mods/app/starter_controller.rs");
pub const STARTER_SERVICE_TEMPLATE: &str =
	include_str!("../../template_sources/src/mods/app/starter_service.rs");

pub const BASIC_MAIN_TEMPLATE: &str = include_str!("../../template_sources/src/basic_main.rs");
pub const BASIC_MODULES_MOD_TEMPLATE: &str = include_str!("../../template_sources/src/mods/basic_mod.rs");
