pub const CONTROLLER_TEMPLATE: &str = r#"use murgamu::prelude::*;
use crate::mods::app::AppService;

#[derive(Clone)]
pub struct AppController {
	service: Arc<AppService>,
}

#[controller("/")]
impl AppController {
	pub fn new(service: Arc<AppService>) -> Self {
		Self { service }
	}

	#[get("/")]
	async fn index(&self) -> MurRes {
		let p_name = mur_env("CARGO_PKG_NAME")?;
		mur_json!(self.service.greet(p_name))
	}

	#[get("/user/:id")]
	async fn get_user(&self, id: Param<u32>) -> MurRes {
		mur_json!(self.service.get_user(*id)) // here you can use id.0 or id.into_inner() too
	}

	/// you can also use queries
	#[get("/query")]
	async fn greet(&self, query: MurQuery<GreetQuery>) -> MurRes {
		let name = query.name.as_deref().unwrap_or("Guest");
		let greeting = self.service.greet(name.to_string());
		mur_json!({ "greeting": greeting })
	}
}

#[derive(Debug, Deserialize, Default)]
pub struct GreetQuery {
	pub name: Option<String>,
}
"#;
