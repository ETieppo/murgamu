use murgamu::prelude::*;
use murgamu::MurRes;
use super::AppService;

pub struct AppController {
  service: AppService,
}

#[controller]
impl AppController {
  fn new(service: AppService) -> Self { 
    Self {
      service
    } 
  }

  #[public]
  #[get]
  async fn check_alive(&self) -> MurRes {
    mur_json!(self.service.is_alive())
  }
}

