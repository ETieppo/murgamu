use super::MurResponseBuilder;
use crate::server::aliases::MurRes;
use http::StatusCode;
use serde::Serialize;

/// Factory for creating HTTP responses by status code.
///
/// `MurHttpResponse` provides a comprehensive set of static methods that
/// return a [`MurResponseBuilder`] pre-configured with the appropriate
/// HTTP status code. Call a body method (`.json()`, `.text()`, `.html()`,
/// `.empty()`) on the builder to produce the final [`MurRes`].
///
/// # Example
///
/// ```rust,ignore
/// return MurHttpResponse::ok().json(serde_json::json!({ "status": "ok" }));
/// return MurHttpResponse::not_found().json(serde_json::json!({ "error": "user not found" }));
/// return MurHttpResponse::no_content();
/// ```
pub struct MurHttpResponse;

impl MurHttpResponse {
	/// `200 OK` — standard successful response.
	pub fn ok() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::OK)
	}

	/// `201 Created` — resource was created successfully.
	pub fn created() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::CREATED)
	}

	/// `202 Accepted` — request accepted for asynchronous processing.
	pub fn accepted() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::ACCEPTED)
	}

	/// `204 No Content` — success with no response body.
	pub fn no_content() -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::NO_CONTENT)
			.empty()
	}

	/// `206 Partial Content` — for range or chunked responses.
	pub fn partial_content() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::PARTIAL_CONTENT)
	}

	/// `301 Moved Permanently` — permanent redirect to `location`.
	pub fn moved_permanently(location: impl Into<String>) -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::MOVED_PERMANENTLY)
			.redirect(location)
	}

	/// `302 Found` — temporary redirect to `location`.
	pub fn found(location: impl Into<String>) -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::FOUND)
			.redirect(location)
	}

	/// `303 See Other` — redirect to `location` using GET.
	pub fn see_other(location: impl Into<String>) -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::SEE_OTHER)
			.redirect(location)
	}

	/// `304 Not Modified` — resource unchanged; client cache is valid.
	pub fn not_modified() -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::NOT_MODIFIED)
			.empty()
	}

	/// `307 Temporary Redirect` — temporary redirect preserving the HTTP method.
	pub fn temporary_redirect(location: impl Into<String>) -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::TEMPORARY_REDIRECT)
			.redirect(location)
	}

	/// `308 Permanent Redirect` — permanent redirect preserving the HTTP method.
	pub fn permanent_redirect(location: impl Into<String>) -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::PERMANENT_REDIRECT)
			.redirect(location)
	}

	/// `400 Bad Request` — the request is malformed or contains invalid data.
	pub fn bad_request() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::BAD_REQUEST)
	}

	/// `401 Unauthorized` — authentication is required.
	pub fn unauthorized() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::UNAUTHORIZED)
	}

	/// `403 Forbidden` — the caller is authenticated but lacks permission.
	pub fn forbidden() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::FORBIDDEN)
	}

	/// `404 Not Found` — the requested resource does not exist.
	pub fn not_found() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::NOT_FOUND)
	}

	/// `405 Method Not Allowed` — the HTTP method is not supported for this route.
	pub fn method_not_allowed() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::METHOD_NOT_ALLOWED)
	}

	/// `406 Not Acceptable` — no content satisfies the `Accept` header.
	pub fn not_acceptable() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::NOT_ACCEPTABLE)
	}

	/// `408 Request Timeout` — the client did not send a request in time.
	pub fn request_timeout() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::REQUEST_TIMEOUT)
	}

	/// `409 Conflict` — the request conflicts with the current server state.
	pub fn conflict() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::CONFLICT)
	}

	/// `410 Gone` — the resource existed but has been permanently removed.
	pub fn gone() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::GONE)
	}

	/// `413 Payload Too Large` — the request body exceeds the allowed size.
	pub fn payload_too_large() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::PAYLOAD_TOO_LARGE)
	}

	/// `415 Unsupported Media Type` — the request `Content-Type` is not supported.
	pub fn unsupported_media_type() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
	}

	/// `422 Unprocessable Entity` — the request is well-formed but semantically invalid.
	pub fn unprocessable_entity() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::UNPROCESSABLE_ENTITY)
	}

	/// `429 Too Many Requests` — the caller has exceeded the rate limit.
	pub fn too_many_requests() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::TOO_MANY_REQUESTS)
	}

	/// `500 Internal Server Error` — an unexpected server-side error occurred.
	pub fn internal_error() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::INTERNAL_SERVER_ERROR)
	}

	/// `501 Not Implemented` — the server does not support the requested feature.
	pub fn not_implemented() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::NOT_IMPLEMENTED)
	}

	/// `502 Bad Gateway` — an upstream service returned an invalid response.
	pub fn bad_gateway() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::BAD_GATEWAY)
	}

	/// `503 Service Unavailable` — the server is temporarily unable to handle requests.
	pub fn service_unavailable() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::SERVICE_UNAVAILABLE)
	}

	/// `504 Gateway Timeout` — an upstream service did not respond in time.
	pub fn gateway_timeout() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::GATEWAY_TIMEOUT)
	}

	/// Returns a builder pre-configured with the given `status` code.
	pub fn status(status: StatusCode) -> MurResponseBuilder {
		MurResponseBuilder::new().status(status)
	}

	/// Shorthand for `200 OK` with a JSON body.
	pub fn json<T: Serialize>(body: T) -> MurRes {
		Self::ok().json(body)
	}

	/// Shorthand for `200 OK` with a plain-text body.
	pub fn text(body: impl Into<String>) -> MurRes {
		Self::ok().text(body)
	}

	/// Shorthand for `200 OK` with an HTML body.
	pub fn html(body: impl Into<String>) -> MurRes {
		Self::ok().html(body)
	}

	/// Shorthand for `400 Bad Request` with a JSON `{ "error": "..." }` body.
	pub fn error(message: impl Into<String>) -> MurRes {
		Self::bad_request().json(serde_json::json!({
			"error": message.into()
		}))
	}

	/// Shorthand for a `302 Found` redirect to `location`.
	pub fn redirect(location: impl Into<String>) -> MurRes {
		Self::found(location)
	}
}
