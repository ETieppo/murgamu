use super::error::MurError;
use super::http::MurRequestContext;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type MurMainResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type MurRes = Result<Response<Full<Bytes>>, MurError>;
pub type MurReq = MurRequestContext;
pub type MurResInfallible = Result<Response<Full<Bytes>>, std::convert::Infallible>;
pub type MurFuture = Pin<Box<dyn Future<Output = MurRes> + Send + 'static>>;
pub type MurRequest = Request<Incoming>;
pub type MurResponse = Response<Full<Bytes>>;
pub type MurRouteHandler = Arc<dyn Fn(MurRequestContext) -> MurFuture + Send + Sync + 'static>;
pub type MurPathParams = HashMap<String, String>;
pub type MurQueryParams = HashMap<String, String>;

#[macro_export]
macro_rules! mur_controllers {
	() => {
		Vec::new()
	};
	($($controller:expr),+ $(,)?) => {
		vec![$(Box::new($controller) as Box<dyn $crate::server::controller::MurController>),+]
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
				$service as std::sync::Arc<dyn $crate::server::service::MurService>
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
			svc as std::sync::Arc<dyn $crate::server::service::MurService>,
		)
	}};
}
