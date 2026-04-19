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

pub const FORMATTER_TEMPLATE: &str = include_str!("../../../sandbox/.rustfmt.toml");
pub const MAIN_TEMPLATE: &str = include_str!("../../../sandbox/src/main.rs");
pub const MODULES_MOD_TEMPLATE: &str = include_str!("../../../sandbox/src/mods/mod.rs");
pub const APP_MOD_CONTENT_TEMPLATE: &str = include_str!("../../../sandbox/src/mods/app/mod.rs");
pub const CONTROLLER_TEMPLATE: &str =
    include_str!("../../../sandbox/src/mods/app/app_controller.rs");
pub const SERVICE_TEMPLATE: &str = include_str!("../../../sandbox/src/mods/app/app_service.rs");
pub const MODELS_MOD_TEMPLATE: &str =
    include_str!("../../../sandbox/src/mods/app/models/mod.rs");
pub const USER_PROPS_TEMPLATE: &str =
    include_str!("../../../sandbox/src/mods/app/models/user_props.rs");
pub const AUTH_MOD_TEMPLATE: &str = include_str!("../../../sandbox/src/mods/auth/mod.rs");
pub const AUTH_GUARD_TEMPLATE: &str = include_str!("../../../sandbox/src/mods/auth/guard.rs");
pub const AUTH_PIPE_TEMPLATE: &str =
    include_str!("../../../sandbox/src/mods/auth/jwt_extraction_pipe.rs");
pub const TOKEN_MOD_TEMPLATE: &str = include_str!("../../../sandbox/src/mods/token/mod.rs");
pub const TOKEN_SERVICE_TEMPLATE: &str =
    include_str!("../../../sandbox/src/mods/token/service.rs");
pub const TOKEN_MODELS_MOD_TEMPLATE: &str =
    include_str!("../../../sandbox/src/mods/token/models/mod.rs");
pub const TOKEN_JWT_TEMPLATE: &str =
    include_str!("../../../sandbox/src/mods/token/models/jwt.rs");
pub const USERS_MOD_TEMPLATE: &str = include_str!("../../../sandbox/src/mods/users/mod.rs");
pub const USERS_ROLE_TEMPLATE: &str = include_str!("../../../sandbox/src/mods/users/role.rs");
