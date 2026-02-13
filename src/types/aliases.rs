use crate::core::error::MurError;
use crate::mur_http::request::MurRequestContext;
use crate::traits::{MurController, MurService};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type MurMainResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type MurRes = Result<Response<Full<Bytes>>, MurError>;
pub type MurResInfallible = Result<Response<Full<Bytes>>, std::convert::Infallible>;
pub type MurFuture = Pin<Box<dyn Future<Output = MurRes> + Send>>;
pub type MurRequest = Request<Incoming>;
pub type MurResponse = Response<Full<Bytes>>;
pub type MurRouteHandler = Arc<dyn Fn(MurRequestContext) -> MurFuture + Send + Sync>;
pub type MurPathParams = HashMap<String, String>;
pub type MurQueryParams = HashMap<String, String>;
pub type MurControllers = Vec<Arc<dyn MurController>>;

pub trait IntoController {
	fn into_controller(self) -> Arc<dyn MurController>;
}

impl<T: MurController + 'static> IntoController for T {
	fn into_controller(self) -> Arc<dyn MurController> {
		Arc::new(self)
	}
}

impl IntoController for Arc<dyn MurController> {
	fn into_controller(self) -> Arc<dyn MurController> {
		self
	}
}

pub fn controllers<I, C>(items: I) -> MurControllers
where
	I: IntoIterator<Item = C>,
	C: IntoController,
{
	items.into_iter().map(|c| c.into_controller()).collect()
}

pub type MurServices = Vec<(std::any::TypeId, Arc<dyn MurService>)>;

#[macro_export]
macro_rules! mur_controllers {
	() => {
		Vec::new()
	};
	($($controller:expr),+ $(,)?) => {
		vec![$(Box::new($controller) as Box<dyn $crate::traits::controller::MurController>),+]
	};
}

#[macro_export]
macro_rules! mur_services {
	() => {
		Vec::new()
	};
	($($service:expr),+ $(,)?) => {
		vec![$(
			(
				std::any::TypeId::of::<std::sync::Arc<_>>(),
				$service as std::sync::Arc<dyn $crate::traits::MurService>
			)
		),+]
	};
}

#[macro_export]
macro_rules! mur_service {
	($service:expr) => {{
		let svc = $service.clone();
		(
			std::any::TypeId::of::<std::sync::Arc<_>>(),
			svc as std::sync::Arc<dyn $crate::traits::MurService>,
		)
	}};
}

#[derive(Debug, Clone)]
pub struct MurRouteInfo {
	pub method: String,
	pub path: String,
	pub controller: String,
	pub handler: String,
}
