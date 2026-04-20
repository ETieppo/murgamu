use super::MurHttpResponse;
use crate::server::aliases::MurRes;
use http::StatusCode;
use serde::Serialize;

/// Extension trait that adds response-building shortcuts directly to [`MurRes`].
///
/// This trait is implemented on `MurRes` (i.e. `Result<Response, MurError>`) so
/// you can write `MurRes::ok()`, `MurRes::json(...)`, etc. instead of
/// `MurHttpResponse::ok().empty()`.
///
/// All methods are imported automatically through `use murgamu::prelude::*`.
pub trait MurResExt {
	/// `200 OK` with an empty body.
	fn ok() -> MurRes;
	/// `201 Created` with an empty body.
	fn created() -> MurRes;
	/// `202 Accepted` with an empty body.
	fn accepted() -> MurRes;
	/// `204 No Content`.
	fn no_content() -> MurRes;
	/// `400 Bad Request` with a JSON error body.
	fn bad_request() -> MurRes;
	/// `401 Unauthorized` with a JSON error body.
	fn unauthorized() -> MurRes;
	/// `403 Forbidden` with a JSON error body.
	fn forbidden() -> MurRes;
	/// `404 Not Found` with a JSON error body.
	fn not_found() -> MurRes;
	/// `500 Internal Server Error` with a JSON error body.
	fn internal_error() -> MurRes;
	/// Empty response with the given `status` code.
	fn status(status: StatusCode) -> MurRes;
	/// `200 OK` with `body` serialized as JSON.
	fn json<T: Serialize>(body: T) -> MurRes;
	/// `200 OK` with `body` as plain text.
	fn text(body: impl Into<String>) -> MurRes;
	/// `200 OK` with `body` as HTML.
	fn html(body: impl Into<String>) -> MurRes;
	/// Redirect to `location` with `302 Found`.
	fn redirect(location: impl Into<String>) -> MurRes;
	/// `400 Bad Request` with `{ "error": message }`.
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
