use crate::server::error::MurError;
use crate::{MurInjects, MurRequestContext, MurServiceContainer};
use std::any::Any;
use std::future::Future;
use std::pin::Pin;

type MurFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;

pub trait MurPipe<Input>: Send + Sync + 'static {
	type Output;
	type Error: Into<MurError>;

	fn apply_transform(
		&self,
		ctx: MurRequestContext,
		input: Input,
	) -> Result<Self::Output, Self::Error>;

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}

	fn as_any(&self) -> &dyn Any;
}

pub trait MurPipeFactory: Send + Sync + 'static {
	fn create(injects: &MurInjects, container: &MurServiceContainer) -> Self
	where
		Self: Sized;
}

pub trait MurPipeDyn: Send + Sync + 'static {
	fn name(&self) -> &str;
	fn as_any(&self) -> &dyn std::any::Any;
}

pub trait MurPipeAsync<Input>: Send + Sync + 'static {
	type Output;
	type Error: Into<MurError>;

	fn apply_transform(
		&self,
		ctx: MurRequestContext,
		input: Input,
	) -> MurFuture<Self::Output, Self::Error>;

	fn name(&self) -> &str {
		std::any::type_name::<Self>()
	}
}
