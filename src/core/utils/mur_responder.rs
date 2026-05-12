use crate::server::aliases::MurRes;
use crate::server::http::MurHttpResponse;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use serde::Serialize;

pub struct MurResponder;

impl MurResponder {
	pub fn json<T: Serialize>(data: T) -> MurRes {
		MurHttpResponse::json(data)
	}

	pub fn text(text: &str) -> MurRes {
		MurHttpResponse::text(text)
	}

	pub fn error(message: &str) -> MurRes {
		MurHttpResponse::bad_request().text(message)
	}

	pub fn status(status: StatusCode, message: &str) -> MurRes {
		MurHttpResponse::status(status).text(message)
	}

	pub fn json_status<T: Serialize>(status: StatusCode, data: T) -> MurRes {
		MurHttpResponse::status(status).json(data)
	}

	pub fn html(html: &str) -> MurRes {
		MurHttpResponse::ok().html(html)
	}

	pub fn redirect(location: &str) -> MurRes {
		if Self::is_dangerous_scheme(location) {
			return MurHttpResponse::bad_request().text("Invalid redirect location");
		}
		match Response::builder()
			.status(StatusCode::FOUND)
			.header("Location", location)
			.body(Full::new(Bytes::new()))
		{
			Ok(resp) => MurRes::from(resp),
			Err(_) => MurHttpResponse::bad_request().text("Invalid redirect location"),
		}
	}

	pub fn redirect_permanent(location: &str) -> MurRes {
		if Self::is_dangerous_scheme(location) {
			return MurHttpResponse::bad_request().text("Invalid redirect location");
		}
		match Response::builder()
			.status(StatusCode::MOVED_PERMANENTLY)
			.header("Location", location)
			.body(Full::new(Bytes::new()))
		{
			Ok(resp) => MurRes::from(resp),
			Err(_) => MurHttpResponse::bad_request().text("Invalid redirect location"),
		}
	}

	fn is_dangerous_scheme(location: &str) -> bool {
		let lower = location.to_ascii_lowercase();
		lower.starts_with("javascript:")
			|| lower.starts_with("data:")
			|| lower.starts_with("vbscript:")
	}

	pub fn no_content() -> MurRes {
		MurHttpResponse::no_content()
	}

	pub fn not_found(message: &str) -> MurRes {
		MurHttpResponse::not_found().json(serde_json::json!({
			"error": "Not Found",
			"message": message,
			"status": 404
		}))
	}

	pub fn unauthorized(message: &str) -> MurRes {
		MurHttpResponse::unauthorized().json(serde_json::json!({
			"error": "Unauthorized",
			"message": message,
			"status": 401
		}))
	}

	pub fn forbidden(message: &str) -> MurRes {
		MurHttpResponse::forbidden().json(serde_json::json!({
			"error": "Forbidden",
			"message": message,
			"status": 403
		}))
	}

	pub fn internal_error(message: &str) -> MurRes {
		MurHttpResponse::internal_error().json(serde_json::json!({
			"error": "Internal Server Error",
			"message": message,
			"status": 500
		}))
	}

	pub fn response_with_headers(
		status: StatusCode,
		body: &str,
		headers: Vec<(&str, &str)>,
	) -> MurRes {
		let mut builder = Response::builder()
			.status(status)
			.header("Content-Type", "text/plain");

		for (name, value) in headers {
			builder = builder.header(name, value);
		}

		match builder.body(Full::new(Bytes::from(body.to_string()))) {
			Ok(resp) => MurRes::from(resp),
			Err(_) => MurHttpResponse::internal_error().text("Failed to build response"),
		}
	}
}
