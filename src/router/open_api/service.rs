use crate::{router::open_api::spec::MurOpenApiSpec, traits::MurService};
use std::sync::Arc;

#[derive(Clone)]
pub struct MurOpenApiService {
	spec: Arc<MurOpenApiSpec>,
	json_cache: Arc<String>,
}

impl MurOpenApiService {
	pub fn new(spec: MurOpenApiSpec) -> Self {
		let json = serde_json::to_string_pretty(&spec).unwrap_or_default();
		Self {
			spec: Arc::new(spec),
			json_cache: Arc::new(json),
		}
	}

	pub fn spec(&self) -> &MurOpenApiSpec {
		&self.spec
	}

	pub fn json(&self) -> &str {
		&self.json_cache
	}
}

impl MurService for MurOpenApiService {
	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
}
