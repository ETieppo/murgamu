use crate::server::error::MurError;
use crate::server::http::response::mur_res::MurRes;
use http::StatusCode;
use http_body_util::Full;
use hyper::Response;
use hyper::body::Bytes;

pub trait MurIntoResponse {
	fn into_response(self) -> MurRes;
}

impl MurIntoResponse for MurRes {
	fn into_response(self) -> MurRes {
		self
	}
}

impl<T: MurIntoResponse> MurIntoResponse for Result<T, MurError> {
	fn into_response(self) -> MurRes {
		match self {
			Ok(v) => v.into_response(),
			Err(e) => MurRes::from(e),
		}
	}
}

impl MurIntoResponse for Response<Full<Bytes>> {
	fn into_response(self) -> MurRes {
		MurRes::from(self)
	}
}

impl MurIntoResponse for String {
	fn into_response(self) -> MurRes {
		MurRes::text(self)
	}
}

impl MurIntoResponse for &str {
	fn into_response(self) -> MurRes {
		MurRes::text(self)
	}
}

impl MurIntoResponse for () {
	fn into_response(self) -> MurRes {
		MurRes::no_content()
	}
}

impl MurIntoResponse for StatusCode {
	fn into_response(self) -> MurRes {
		MurRes::from(
			Response::builder()
				.status(self)
				.body(Full::new(Bytes::new()))
				.unwrap(),
		)
	}
}

impl<T: MurIntoResponse> MurIntoResponse for (StatusCode, T) {
	fn into_response(self) -> MurRes {
		let (status, body) = self;
		body.into_response().with_status(status)
	}
}

impl MurIntoResponse for serde_json::Value {
	fn into_response(self) -> MurRes {
		MurRes::json(self)
	}
}

impl MurIntoResponse for Bytes {
	fn into_response(self) -> MurRes {
		MurRes::bytes(self)
	}
}

impl MurIntoResponse for Vec<u8> {
	fn into_response(self) -> MurRes {
		MurRes::bytes(Bytes::from(self))
	}
}
