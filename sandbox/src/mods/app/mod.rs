mod app_controller;
mod app_service;
mod models;

pub use app_controller::AppController;
pub use app_service::AppService;
use murgamu::prelude::*;

#[module(
  controllers=[AppController],
  services=[AppService]
)]
pub struct AppModule;
