use super::MurConfigProviderBuilder;
use super::MurConfigService;
use crate::config::MurConfig;
use std::sync::Arc;

#[derive(Clone)]
pub struct MurConfigProvider {
	pub builder_fn: Option<Arc<dyn Fn() -> MurConfig + Send + Sync>>,
	pub config: Option<MurConfig>,
}

impl std::fmt::Debug for MurConfigProvider {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurConfigProvider")
			.field("builder_fn", &self.builder_fn.as_ref().map(|_| "<fn>"))
			.field("config", &self.config)
			.finish()
	}
}

impl MurConfigProvider {
	pub fn from_env() -> Self {
		Self {
			builder_fn: None,
			config: Some(MurConfig::from_env()),
		}
	}

	pub fn from_config(config: MurConfig) -> Self {
		Self {
			builder_fn: None,
			config: Some(config),
		}
	}

	pub fn with_builder<F>(builder: F) -> Self
	where
		F: Fn() -> MurConfig + Send + Sync + 'static,
	{
		Self {
			builder_fn: Some(Arc::new(builder)),
			config: None,
		}
	}

	pub fn builder() -> MurConfigProviderBuilder {
		MurConfigProviderBuilder::new()
	}

	pub fn provide(&self) -> MurConfigService {
		if let Some(config) = &self.config {
			MurConfigService::new(config.clone())
		} else if let Some(builder) = &self.builder_fn {
			MurConfigService::new(builder())
		} else {
			MurConfigService::from_env()
		}
	}
}

impl Default for MurConfigProvider {
	fn default() -> Self {
		MurConfigProvider::from_env()
	}
}
