mod attributes;
mod manual;

use manual::ModuleTest;
use murgamu::MurMainResult;
use murgamu::MurServer;
use murgamu::tokio;

#[tokio::main]
async fn main() -> MurMainResult {
	MurServer::new()
		.module(ModuleTest::new())
		.bind(("127.0.0.1", 3000))?
		.run()
		.await
}
