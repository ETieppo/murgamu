mod mods;

use murgamu::MurMainResult;
use murgamu::MurServer;
use murgamu::prelude::*;

#[murgamu::main]
async fn main() -> MurMainResult {
	MurServer::new()
		.bind("127.0.0.1:3000")?
		.run()
		.await
}

