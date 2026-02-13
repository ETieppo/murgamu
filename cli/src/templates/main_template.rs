pub const MAIN_TEMPLATE: &str = r#"mod mods;

use mods::app::AppModule;
use murgamu::prelude::*;

#[murgamu::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	println!("🚀 Murgamü Server");
	println!("   GET http://localhost:3000/");
	println!("   GET http://localhost:3000/greet?name=World\n");

	MurServer::new()
		.module(AppModule::new())
		.bind("127.0.0.1:3000")?
		.run()
		.await
}
"#;
