use crate::server::http::response::mur_res::MurRes;
use http::StatusCode;
use serde::Serialize;

/// Extension trait that adds response-building shortcuts as static methods on [`MurRes`].
/// Most of these are now available directly on [`MurRes`]; this trait is kept for
/// backwards-compatibility with code that uses `use murgamu::prelude::*`.
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
		MurRes::no_content()
	}

	fn created() -> MurRes {
		MurRes::from(
			crate::server::http::response::builder::MurResponseBuilder::new()
				.status(StatusCode::CREATED)
				.empty(),
		)
	}

	fn accepted() -> MurRes {
		MurRes::from(
			crate::server::http::response::builder::MurResponseBuilder::new()
				.status(StatusCode::ACCEPTED)
				.empty(),
		)
	}

	fn no_content() -> MurRes {
		MurRes::no_content()
	}

	fn bad_request() -> MurRes {
		MurRes::bad_request(serde_json::json!({ "error": "Bad Request" }))
	}

	fn unauthorized() -> MurRes {
		MurRes::unauthorized(serde_json::json!({ "error": "Unauthorized" }))
	}

	fn forbidden() -> MurRes {
		MurRes::forbidden(serde_json::json!({ "error": "Forbidden" }))
	}

	fn not_found() -> MurRes {
		MurRes::not_found(serde_json::json!({ "error": "Not Found" }))
	}

	fn internal_error() -> MurRes {
		MurRes::internal_error(serde_json::json!({ "error": "Internal Server Error" }))
	}

	fn status(status: StatusCode) -> MurRes {
		MurRes::from(
			crate::server::http::response::builder::MurResponseBuilder::new()
				.status(status)
				.empty(),
		)
	}

	fn json<T: Serialize>(body: T) -> MurRes {
		MurRes::json(body)
	}

	fn text(body: impl Into<String>) -> MurRes {
		MurRes::text(body)
	}

	fn html(body: impl Into<String>) -> MurRes {
		MurRes::html(body)
	}

	fn redirect(location: impl Into<String>) -> MurRes {
		MurRes::redirect(location)
	}

	fn error(message: impl Into<String>) -> MurRes {
		MurRes::bad_request(serde_json::json!({ "error": message.into() }))
	}
}
