use crate::core::error::MurError;
use crate::mur_http::request::MurRequestContext;
use crate::MurRes;

pub trait MurExceptionFilter: Send + Sync + 'static {
	fn catch(&self, error: MurError, ctx: &MurRequestContext) -> MurRes;

	fn can_handle(&self, _error: &MurError) -> bool {
		true
	}

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}
