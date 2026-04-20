mod models;
mod service;

use murgamu::prelude::*;
pub use service::TokenService;

#[module(
	providers: [TokenService] 
	exports: [TokenService]
)]
pub struct TokenModule;

