use crate::core::error::MurError;
use std::future::Future;
use std::pin::Pin;

type MurFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;

pub trait MurPipe<Input>: Send + Sync + 'static {
	type Output;
	type Error: Into<MurError>;

	fn transform(&self, input: Input) -> Result<Self::Output, Self::Error>;

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}

pub trait MurPipeAsync<Input>: Send + Sync + 'static {
	type Output;
	type Error: Into<MurError>;

	fn transform(&self, input: Input) -> MurFuture<Self::Output, Self::Error>;

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}
