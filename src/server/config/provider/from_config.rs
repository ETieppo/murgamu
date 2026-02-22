use crate::server::config::MurConfig;
use crate::server::config::MurConfigError;

pub trait MurFromConfig: Sized {
	fn from_config(config: &MurConfig) -> Result<Self, MurConfigError>;

	fn from_config_prefix(config: &MurConfig, prefix: &str) -> Result<Self, MurConfigError> {
		let subset = config.subset(prefix);
		Self::from_config(&subset)
	}
}
