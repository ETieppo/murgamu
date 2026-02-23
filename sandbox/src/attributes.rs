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
}

pub struct ControllerTest;

#[controller("test")]
impl ControllerTest {
	pub fn new(s: Arc<ServiceTest>) -> Self {
		Self
	}
}
