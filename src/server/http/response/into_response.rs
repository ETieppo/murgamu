use super::MurHttpResponse;
use crate::server::aliases::MurRes;
use crate::server::error::MurError;
use http::StatusCode;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Response;

pub trait MurIntoResponse {
	fn into_response(self) -> MurRes;
}

impl<T: MurIntoResponse> MurIntoResponse for Result<T, MurError> {
	fn into_response(self) -> MurRes {
		match self {
			Ok(v) => v.into_response(),
			Err(e) => e.into_response(),
		}
	}
}

impl MurIntoResponse for Response<Full<Bytes>> {
	fn into_response(self) -> MurRes {
		Ok(self)
	}
}

impl MurIntoResponse for String {
	fn into_response(self) -> MurRes {
		MurHttpResponse::text(self)
	}
}

impl MurIntoResponse for &str {
	fn into_response(self) -> MurRes {
		MurHttpResponse::text(self)
	}
}

impl MurIntoResponse for () {
	fn into_response(self) -> MurRes {
		MurHttpResponse::no_content()
	}
}

impl MurIntoResponse for StatusCode {
	fn into_response(self) -> MurRes {
		MurHttpResponse::status(self).empty()
	}
}

impl<T: MurIntoResponse> MurIntoResponse for (StatusCode, T) {
	fn into_response(self) -> MurRes {
		let (status, body) = self;

		let response = body.into_response()?;
		let (parts, body) = response.into_parts();
		let mut new_response = Response::from_parts(parts, body);
		*new_response.status_mut() = status;
		Ok(new_response)
	}
}

impl MurIntoResponse for serde_json::Value {
	fn into_response(self) -> MurRes {
		MurHttpResponse::json(self)
	}
}

impl MurIntoResponse for Bytes {
	fn into_response(self) -> MurRes {
		MurHttpResponse::ok().bytes(self)
	}
}

impl MurIntoResponse for Vec<u8> {
	fn into_response(self) -> MurRes {
		MurHttpResponse::ok().bytes(Bytes::from(self))
	}
}
