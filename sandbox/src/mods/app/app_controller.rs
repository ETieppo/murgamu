use crate::mods::app::AppService;
use murgamu::prelude::*;

#[derive(Clone)]
pub struct AppController {
	service: AppService,
}

#[controller("/")]
impl AppController {
	pub const fn new(service: AppService) -> Self {
		Self { service }
	}

	#[get("/")]
	async fn index(&self) -> MurRes {
		mur_json!(self.service.greet().await?)
	}

	#[get("/user/:id")]
	async fn get_user(&self, #[param] id: u32) -> MurRes {
		// here you can use id.0 or id.into_inner() too
		mur_json!(self.service.get_user(id))
	}

	/// you can also use queries
	#[get("/query")]
	async fn greet(&self, query: MurQuery<GreetQuery>) -> MurRes {
		let name = query.name.as_deref().unwrap_or("Guest");
		let greeting = self.service.greet().await?;
		mur_json!({"greeting": greeting,  "name": name })
	}

	#[public]
	#[post("/create")]
	async fn receive_body(&self, #[body] b: BodyTest) -> MurRes {
		mur_json!(format!("body received {b:?}"))
	}
}

#[derive(Debug, Deserialize, Default)]
pub struct GreetQuery {
	pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BodyTest {
	username: String,
	password: String,
	email: String,
}
