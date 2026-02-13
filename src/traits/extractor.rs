use crate::core::error::MurError;
use crate::mur_http::request::MurRequestContext;
use std::future::Future;
use std::pin::Pin;

pub trait MurExtractor: Sized + Send {
	type Error: Into<MurError>;

	fn extract(
		ctx: &MurRequestContext,
	) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + '_>>;
}

pub trait MurExtractorSync: Sized + Send {
	type Error: Into<MurError>;

	fn extract_sync(ctx: &MurRequestContext) -> Result<Self, Self::Error>;
}
