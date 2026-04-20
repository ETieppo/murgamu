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

  #[get]
  async fn check_alive(&self) -> MurRes {
    self.service.is_alive()
  }
}
