use crate::mods::app::AppService;
use murgamu::Param;
use murgamu::prelude::*;

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
		mur_json!(self.service.greet()?)
	}

	#[get("/user/:id")]
	async fn get_user(&self, id: Param<u32>) -> MurRes {
		// here you can use id.0 or id.into_inner() too
		mur_json!(self.service.get_user(*id))
	}

	/// you can also use queries
	#[get("/query")]
	async fn greet(&self, query: MurQuery<GreetQuery>) -> MurRes {
		let name = query.name.as_deref().unwrap_or("Guest");
		let greeting = self.service.greet()?;
		mur_json!({"greeting": greeting,  "name": name })
	}

	#[post("/create")]
	async fn receive_body(&self, b:MurBody) -> MurRes {
		println!("{b:?}");
		mur_json!("body received")
	}
}

#[derive(Debug, Deserialize, Default)]
pub struct GreetQuery {
	pub name: Option<String>,
}
