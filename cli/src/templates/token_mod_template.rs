pub const TOKEN_MOD_TEMPLATE: &str = r#"mod models;
mod service;

use murgamu::prelude::*;
pub use service::TokenService;

#[module(
	services=[TokenService]
)]
pub struct TokenModule;
"#;
