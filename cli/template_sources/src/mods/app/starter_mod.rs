mod controller;
mod service;

pub use controller::AppController;
pub use service::AppService;
use murgamu::prelude::*;

#[module(
  controllers: [AppController]
  providers: [AppService]
)]
pub struct AppModule;
