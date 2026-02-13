use super::service::MurOpenApiService;
use super::spec::MurOpenApiSpec;
use super::swagger::MurSwagger;
use crate::container::core::MurServiceContainer;
use crate::mur_http::request::MurRequestContext;
use crate::traits::MurController;
use crate::types::{MurHttpResponse, MurRouteHandler};
use std::sync::Arc;

#[derive(Clone)]
pub struct MurOpenApiController {
	service: Arc<MurOpenApiService>,
	path_prefix: String,
}

impl MurOpenApiController {
	pub fn new(spec: MurOpenApiSpec) -> Self {
		Self {
			service: Arc::new(MurOpenApiService::new(spec)),
			path_prefix: "/api-docs".to_string(),
		}
	}

	pub fn with_path(spec: MurOpenApiSpec, path: impl Into<String>) -> Self {
		Self {
			service: Arc::new(MurOpenApiService::new(spec)),
			path_prefix: path.into(),
		}
	}

	pub fn service(&self) -> Arc<MurOpenApiService> {
		Arc::clone(&self.service)
	}
}

impl MurController for MurOpenApiController {
	fn routes(
		self: Arc<Self>,
		_container: Arc<MurServiceContainer>,
	) -> Vec<(String, String, MurRouteHandler)> {
		let service = Arc::clone(&self.service);
		let service2 = Arc::clone(&self.service);
		let prefix = self.path_prefix.clone();
		let prefix2 = self.path_prefix.clone();

		vec![
			("GET".to_string(), format!("{}/openapi.json", prefix), {
				let svc = service.clone();
				Arc::new(move |_ctx: MurRequestContext| {
					let json = svc.json().to_string();
					Box::pin(async move {
						MurHttpResponse::ok()
							.header("Content-Type", "application/json")
							.text(json)
					})
				})
			}),
			("GET".to_string(), prefix2.to_string(), {
				let _svc = service2.clone();
				let path = prefix2.clone();
				Arc::new(move |_ctx: MurRequestContext| {
					let spec_url = format!("{}/openapi.json", path);
					let html = MurSwagger::generate_ui(&spec_url);
					Box::pin(async move {
						MurHttpResponse::ok()
							.header("Content-Type", "text/html")
							.text(html)
					})
				})
			}),
		]
	}

	fn base_path(&self) -> &str {
		""
	}

	fn name(&self) -> &str {
		"MurOpenApiController"
	}
}
