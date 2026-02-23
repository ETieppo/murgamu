use murgamu::{MurMainResult, MurServer, prelude::*};

#[murgamu::main]
async fn main() -> MurMainResult {
	MurServer::new()
		.module(ModuleTest::new())
		.bind(("127.0.0.1", 3000))?
		.run()
		.await
}

#[module()]
pub struct ModuleX;

#[module(
	imports=[ModuleX],
	controllers=[ControllerTest],
  services=[ServiceTest],
  exports=[ServiceTest]
)]
pub struct ModuleTest;

#[service]
pub struct ServiceTest;
impl ServiceTest {
	pub fn new() -> Self {
		Self
	}

	pub fn get_name(&self) -> &'static str {
		"Mur"
	}
}

pub struct ControllerTest {
	service: Arc<ServiceTest>,
}

#[controller("test")]
impl ControllerTest {
	pub fn new(service: Arc<ServiceTest>) -> Self {
		Self { service }
	}

	#[get("name")]
	async fn get_name(&self) -> MurRes {
		mur_json!(self.service.get_name())
	}
}
