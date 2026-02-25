use crate::MurResponse;
use crate::server::aliases::MurRes;
use http::StatusCode;
use http_body_util::Full;
use hyper::Response;
use hyper::body::Bytes;

pub type MurResult<T> = Result<T, MurError>;

#[derive(Debug)]
pub enum MurError {
	Hyper(hyper::Error),
	Serde(String),
	NotFound(String),
	Unauthorized(String),
	Forbidden(String),
	BadRequest(String),
	Internal(String),
	NoEnv(&'static str),
	PayloadTooLarge(String),
	Custom(StatusCode, String),
}

impl std::fmt::Display for MurError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MurError::Hyper(e) => write!(f, "HTTP error: {}", e),
			MurError::Serde(e) => write!(f, "Serialization error: {}", e),
			MurError::NotFound(e) => write!(f, "Not found: {}", e),
			MurError::Unauthorized(e) => write!(f, "Unauthorized: {}", e),
			MurError::Forbidden(e) => write!(f, "Forbidden: {}", e),
			MurError::BadRequest(e) => write!(f, "Bad request: {}", e),
			MurError::Internal(e) => write!(f, "Internal error: {}", e),
			MurError::NoEnv(e) => write!(f, "No Environment Internal error: {}", e),
			MurError::PayloadTooLarge(e) => write!(f, "Payload too large: {}", e),
			MurError::Custom(status, e) => write!(f, "Error {}: {}", status.as_u16(), e),
		}
	}
}

impl From<Box<dyn std::error::Error + Send + Sync>> for MurError {
	fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
		MurError::internal(err.to_string())
	}
}

impl std::error::Error for MurError {}

impl From<hyper::Error> for MurError {
	fn from(err: hyper::Error) -> Self {
		MurError::Hyper(err)
	}
}

impl From<serde_json::Error> for MurError {
	fn from(err: serde_json::Error) -> Self {
		MurError::Serde(err.to_string())
	}
}

impl From<std::io::Error> for MurError {
	fn from(err: std::io::Error) -> Self {
		MurError::Internal(err.to_string())
	}
}

impl From<std::string::FromUtf8Error> for MurError {
	fn from(err: std::string::FromUtf8Error) -> Self {
		MurError::BadRequest(format!("Invalid UTF-8: {}", err))
	}
}

impl From<String> for MurError {
	fn from(msg: String) -> Self {
		MurError::Internal(msg)
	}
}

impl From<&str> for MurError {
	fn from(msg: &str) -> Self {
		MurError::Internal(msg.to_string())
	}
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for MurError {
	fn from(err: diesel::result::Error) -> Self {
		match err {
			diesel::result::Error::NotFound => MurError::NotFound("Register was not found".into()),
			diesel::result::Error::SerializationError(err) => MurError::BadRequest(err.to_string()),
			diesel::result::Error::RollbackErrorOnCommit { .. } => {
				MurError::internal("Transaction failed irrecoverably")
			}
			diesel::result::Error::RollbackTransaction => {
				MurError::internal("Unexpected manual rollback outside transaction boundary")
			}
			diesel::result::Error::DatabaseError(kind, info) => {
				use diesel::result::DatabaseErrorKind::*;

				match kind {
					UniqueViolation => MurError::conflict(info.message()),
					ForeignKeyViolation => MurError::conflict(info.message()),
					NotNullViolation => MurError::bad_request(info.message()),
					CheckViolation => MurError::bad_request(info.message()),
					_ => MurError::internal(info.message()),
				}
			}
			_ => MurError::internal(format!("Database error: {}", err)),
		}
	}
}

impl MurError {
	pub fn status_code(&self) -> StatusCode {
		match self {
			MurError::Hyper(_) => StatusCode::INTERNAL_SERVER_ERROR,
			MurError::Serde(_) => StatusCode::BAD_REQUEST,
			MurError::NotFound(_) => StatusCode::NOT_FOUND,
			MurError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
			MurError::Forbidden(_) => StatusCode::FORBIDDEN,
			MurError::BadRequest(_) => StatusCode::BAD_REQUEST,
			MurError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
			MurError::NoEnv(_) => StatusCode::INTERNAL_SERVER_ERROR,
			MurError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
			MurError::Custom(status, _) => *status,
		}
	}

	pub fn message(&self) -> &str {
		match self {
			MurError::Hyper(_) => "HTTP transport error",
			MurError::Serde(msg) => msg,
			MurError::NotFound(msg) => msg,
			MurError::Unauthorized(msg) => msg,
			MurError::Forbidden(msg) => msg,
			MurError::BadRequest(msg) => msg,
			MurError::Internal(msg) => msg,
			MurError::NoEnv(msg) => msg,
			MurError::PayloadTooLarge(_) => "Request body exceeds the maximum allowed size",
			MurError::Custom(_, msg) => msg,
		}
	}

	pub fn kind(&self) -> &'static str {
		match self {
			MurError::Hyper(_) => "hyper",
			MurError::Serde(_) => "serde",
			MurError::NotFound(_) => "not_found",
			MurError::Unauthorized(_) => "unauthorized",
			MurError::Forbidden(_) => "forbidden",
			MurError::BadRequest(_) => "bad_request",
			MurError::Internal(_) => "internal",
			MurError::NoEnv(_) => "internal",
			MurError::PayloadTooLarge(_) => "too_large",
			MurError::Custom(_, _) => "custom",
		}
	}

	pub fn not_found(msg: impl Into<String>) -> Self {
		MurError::NotFound(msg.into())
	}

	pub fn bad_request(msg: impl Into<String>) -> Self {
		MurError::BadRequest(msg.into())
	}

	pub fn unauthorized(msg: impl Into<String>) -> Self {
		MurError::Unauthorized(msg.into())
	}

	pub fn forbidden(msg: impl Into<String>) -> Self {
		MurError::Forbidden(msg.into())
	}

	pub fn internal(msg: impl Into<String>) -> Self {
		MurError::Internal(msg.into())
	}

	pub fn validation(msg: impl Into<String>) -> Self {
		MurError::BadRequest(msg.into())
	}

	pub fn custom(status: StatusCode, msg: impl Into<String>) -> Self {
		MurError::Custom(status, msg.into())
	}

	pub fn conflict(msg: impl Into<String>) -> Self {
		MurError::Custom(StatusCode::CONFLICT, msg.into())
	}

	pub fn payload_too_large(msg: impl Into<String>) -> Self {
		MurError::PayloadTooLarge(msg.into())
	}

	pub fn gone(msg: impl Into<String>) -> Self {
		MurError::Custom(StatusCode::GONE, msg.into())
	}

	pub fn unprocessable(msg: impl Into<String>) -> Self {
		MurError::Custom(StatusCode::UNPROCESSABLE_ENTITY, msg.into())
	}

	pub fn too_many_requests(msg: impl Into<String>) -> Self {
		MurError::Custom(StatusCode::TOO_MANY_REQUESTS, msg.into())
	}

	pub fn service_unavailable(msg: impl Into<String>) -> Self {
		MurError::Custom(StatusCode::SERVICE_UNAVAILABLE, msg.into())
	}

	pub fn into_response(self) -> MurResponse {
		let status = self.status_code();
		let kind = self.kind();
		let message = self.to_string();
		let body = serde_json::json!({
			"error": message,
			"status": status.as_u16(),
			"kind": kind
		});

		Response::builder()
			.status(status)
			.header("Content-Type", "application/json")
			.body(Full::new(Bytes::from(
				serde_json::to_string(&body).unwrap_or_else(|_| {
					format!(r#"{{"error":"{}","status":{}}}"#, message, status.as_u16())
				}),
			)))
			.unwrap()
	}

	pub fn into_response_with_context(self, context: serde_json::Value) -> MurRes {
		let status = self.status_code();
		let kind = self.kind();
		let message = self.to_string();
		let body = serde_json::json!({
			"error": message,
			"status": status.as_u16(),
			"kind": kind,
			"context": context
		});

		Ok(Response::builder()
			.status(status)
			.header("Content-Type", "application/json")
			.body(Full::new(Bytes::from(
				serde_json::to_string(&body).unwrap_or_else(|_| {
					format!(r#"{{"error":"{}","status":{}}}"#, message, status.as_u16())
				}),
			)))
			.unwrap())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_error_status_codes() {
		assert_eq!(MurError::not_found("").status_code(), StatusCode::NOT_FOUND);
		assert_eq!(
			MurError::bad_request("").status_code(),
			StatusCode::BAD_REQUEST
		);
		assert_eq!(
			MurError::unauthorized("").status_code(),
			StatusCode::UNAUTHORIZED
		);
		assert_eq!(MurError::forbidden("").status_code(), StatusCode::FORBIDDEN);
		assert_eq!(
			MurError::internal("").status_code(),
			StatusCode::INTERNAL_SERVER_ERROR
		);
		assert_eq!(MurError::conflict("").status_code(), StatusCode::CONFLICT);
	}

	#[test]
	fn test_error_display() {
		let err = MurError::not_found("User not found");
		assert_eq!(err.to_string(), "Not found: User not found");

		let err = MurError::bad_request("Invalid input");
		assert_eq!(err.to_string(), "Bad request: Invalid input");
	}

	#[test]
	fn test_error_kind() {
		assert_eq!(MurError::not_found("").kind(), "not_found");
		assert_eq!(MurError::bad_request("").kind(), "bad_request");
		assert_eq!(MurError::internal("").kind(), "internal");
	}

	#[test]
	fn test_from_string() {
		let err: MurError = "Something went wrong".into();
		assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
	}
}
