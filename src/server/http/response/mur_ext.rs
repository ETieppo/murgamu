use super::MurHttpResponse;
use crate::server::aliases::MurRes;
use http::StatusCode;
use serde::Serialize;

pub trait MurResExt {
	fn ok() -> MurRes;
	fn created() -> MurRes;
	fn accepted() -> MurRes;
	fn no_content() -> MurRes;
	fn bad_request() -> MurRes;
	fn unauthorized() -> MurRes;
	fn forbidden() -> MurRes;
	fn not_found() -> MurRes;
	fn internal_error() -> MurRes;
	fn status(status: StatusCode) -> MurRes;
	fn json<T: Serialize>(body: T) -> MurRes;
	fn text(body: impl Into<String>) -> MurRes;
	fn html(body: impl Into<String>) -> MurRes;
	fn redirect(location: impl Into<String>) -> MurRes;
	fn error(message: impl Into<String>) -> MurRes;
}

impl MurResExt for MurRes {
	fn ok() -> MurRes {
		MurHttpResponse::ok().empty()
	}

	fn created() -> MurRes {
		MurHttpResponse::created().empty()
	}

	fn accepted() -> MurRes {
		MurHttpResponse::accepted().empty()
	}

	fn no_content() -> MurRes {
		MurHttpResponse::no_content()
	}

	fn bad_request() -> MurRes {
		MurHttpResponse::bad_request().json(serde_json::json!({
			"error": "Bad Request"
		}))
	}

	fn unauthorized() -> MurRes {
		MurHttpResponse::unauthorized().json(serde_json::json!({
			"error": "Unauthorized"
		}))
	}

	fn forbidden() -> MurRes {
		MurHttpResponse::forbidden().json(serde_json::json!({
			"error": "Forbidden"
		}))
	}

	fn not_found() -> MurRes {
		MurHttpResponse::not_found().json(serde_json::json!({
			"error": "Not Found"
		}))
	}

	fn internal_error() -> MurRes {
		MurHttpResponse::internal_error().json(serde_json::json!({
			"error": "Internal Server Error"
		}))
	}

	fn status(status: StatusCode) -> MurRes {
		MurHttpResponse::status(status).empty()
	}

	fn json<T: Serialize>(body: T) -> MurRes {
		MurHttpResponse::json(body)
	}

	fn text(body: impl Into<String>) -> MurRes {
		MurHttpResponse::text(body)
	}

	fn html(body: impl Into<String>) -> MurRes {
		MurHttpResponse::html(body)
	}

	fn redirect(location: impl Into<String>) -> MurRes {
		MurHttpResponse::redirect(location)
	}

	fn error(message: impl Into<String>) -> MurRes {
		MurHttpResponse::error(message)
	}
}
