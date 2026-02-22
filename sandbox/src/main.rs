use murgamu::{MurMainResult, MurServer, prelude::*};

#[murgamu::main]
async fn main() -> MurMainResult {
	MurServer::new()
		.module(ModuleTest::new())
		.bind(("127.0.0.1", 3000))?
		.run()
		.await
}

#[module]
pub struct ModuleTest;

#[service]
pub struct ServiceTest;
impl ServiceTest {
	fn new() -> Self {
		Self
	}
}

pub struct ControllerTest;

#[controller("test")]
impl ControllerTest {}
