use super::MurError;
use crate::server::aliases::MurRes;
use crate::server::http::MurRequestContext;

pub trait MurExceptionFilter: Send + Sync + 'static {
	fn catch(&self, error: MurError, ctx: &MurRequestContext) -> MurRes;

	fn can_handle(&self, _error: &MurError) -> bool {
		true
	}

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}
