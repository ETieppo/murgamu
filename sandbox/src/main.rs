mod mods;

use mods::app::AppModule;
use murgamu::MurMainResult;
use murgamu::MurServer;
use murgamu::mur_env;
use murgamu::prelude::*;

#[murgamu::main]
async fn main() -> MurMainResult {
	MurServer::new()
		.service(InjectNameService)
		.module(AppModule::new())
		.bind("127.0.0.1:3000")?
		.run()
		.await
}

#[service]
pub struct InjectNameService;
impl InjectNameService {
	fn new() -> Self {
		Self
	}
	pub fn get_name(&self) -> Result<String, MurError> {
		mur_env("CARGO_PKG_NAME")
	}
}
