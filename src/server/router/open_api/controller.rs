use super::service::MurOpenApiService;
use super::spec::MurOpenApiSpec;
use super::swagger::MurSwagger;
use crate::server::aliases::MurRouteDefinition;
use crate::server::controller::MurController;
use crate::server::http::MurHttpResponse;
use crate::server::http::MurRequestContext;
use crate::server::service::MurServiceContainer;
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
	fn routes(self: Arc<Self>, _container: Arc<MurServiceContainer>) -> Vec<MurRouteDefinition> {
		let service = Arc::clone(&self.service);
		let service2 = Arc::clone(&self.service);
		let prefix = self.path_prefix.clone();
		let prefix2 = self.path_prefix.clone();

		vec![
			MurRouteDefinition {
				method: "GET".to_string(),
				path: format!("{}/openapi.json", prefix),
				handler: {
					let svc = service.clone();
					Arc::new(move |_ctx: MurRequestContext| {
						let json = svc.json().to_string();
						Box::pin(async move {
							MurHttpResponse::ok()
								.header("Content-Type", "application/json")
								.text(json)
						})
					})
				},
				is_public: true,
				allowed_roles: vec![],
			},
			MurRouteDefinition {
				method: "GET".to_string(),
				path: prefix2.to_string(),
				handler: {
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
				},
				is_public: true,
				allowed_roles: vec![],
			},
		]
	}

	fn base_path(&self) -> &str {
		""
	}

	fn name(&self) -> &str {
		"MurOpenApiController"
	}
}
