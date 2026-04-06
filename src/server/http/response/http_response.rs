use super::MurResponseBuilder;
use crate::server::aliases::MurRes;
use http::StatusCode;
use serde::Serialize;

pub struct MurHttpResponse;

impl MurHttpResponse {
	pub fn ok() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::OK)
	}

	pub fn created() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::CREATED)
	}

	pub fn accepted() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::ACCEPTED)
	}

	pub fn no_content() -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::NO_CONTENT)
			.empty()
	}

	pub fn partial_content() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::PARTIAL_CONTENT)
	}

	pub fn moved_permanently(location: impl Into<String>) -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::MOVED_PERMANENTLY)
			.redirect(location)
	}

	pub fn found(location: impl Into<String>) -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::FOUND)
			.redirect(location)
	}

	pub fn see_other(location: impl Into<String>) -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::SEE_OTHER)
			.redirect(location)
	}

	pub fn not_modified() -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::NOT_MODIFIED)
			.empty()
	}

	pub fn temporary_redirect(location: impl Into<String>) -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::TEMPORARY_REDIRECT)
			.redirect(location)
	}

	pub fn permanent_redirect(location: impl Into<String>) -> MurRes {
		MurResponseBuilder::new()
			.status(StatusCode::PERMANENT_REDIRECT)
			.redirect(location)
	}

	pub fn bad_request() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::BAD_REQUEST)
	}

	pub fn unauthorized() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::UNAUTHORIZED)
	}

	pub fn forbidden() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::FORBIDDEN)
	}

	pub fn not_found() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::NOT_FOUND)
	}

	pub fn method_not_allowed() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::METHOD_NOT_ALLOWED)
	}

	pub fn not_acceptable() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::NOT_ACCEPTABLE)
	}

	pub fn request_timeout() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::REQUEST_TIMEOUT)
	}

	pub fn conflict() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::CONFLICT)
	}

	pub fn gone() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::GONE)
	}

	pub fn payload_too_large() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::PAYLOAD_TOO_LARGE)
	}

	pub fn unsupported_media_type() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
	}

	pub fn unprocessable_entity() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::UNPROCESSABLE_ENTITY)
	}

	pub fn too_many_requests() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::TOO_MANY_REQUESTS)
	}

	pub fn internal_error() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::INTERNAL_SERVER_ERROR)
	}

	pub fn not_implemented() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::NOT_IMPLEMENTED)
	}

	pub fn bad_gateway() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::BAD_GATEWAY)
	}

	pub fn service_unavailable() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::SERVICE_UNAVAILABLE)
	}

	pub fn gateway_timeout() -> MurResponseBuilder {
		MurResponseBuilder::new().status(StatusCode::GATEWAY_TIMEOUT)
	}

	pub fn status(status: StatusCode) -> MurResponseBuilder {
		MurResponseBuilder::new().status(status)
	}

	pub fn json<T: Serialize>(body: T) -> MurRes {
		Self::ok().json(body)
	}

	pub fn text(body: impl Into<String>) -> MurRes {
		Self::ok().text(body)
	}

	pub fn html(body: impl Into<String>) -> MurRes {
		Self::ok().html(body)
	}

	pub fn error(message: impl Into<String>) -> MurRes {
		Self::bad_request().json(serde_json::json!({
			"error": message.into()
		}))
	}

	pub fn redirect(location: impl Into<String>) -> MurRes {
		Self::found(location)
	}
}
