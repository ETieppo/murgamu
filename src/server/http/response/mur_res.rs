use crate::server::error::MurError;
use http::StatusCode;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Response;
use serde::Serialize;

pub(crate) type RawRes = Result<Response<Full<Bytes>>, MurError>;

// ─── Cookie ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SameSite {
	Strict,
	Lax,
	None,
}

impl SameSite {
	fn as_str(&self) -> &str {
		match self {
			Self::Strict => "Strict",
			Self::Lax => "Lax",
			Self::None => "None",
		}
	}
}

#[derive(Debug, Clone)]
pub struct MurCookie {
	pub name: String,
	pub value: String,
	pub path: Option<String>,
	pub domain: Option<String>,
	pub max_age: Option<u64>,
	pub secure: bool,
	pub http_only: bool,
	pub same_site: SameSite,
}

impl MurCookie {
	pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			value: value.into(),
			path: None,
			domain: None,
			max_age: None,
			secure: false,
			http_only: false,
			same_site: SameSite::Lax,
		}
	}

	pub fn http_only(mut self) -> Self {
		self.http_only = true;
		self
	}

	pub fn secure(mut self) -> Self {
		self.secure = true;
		self
	}

	pub fn same_site(mut self, same_site: SameSite) -> Self {
		self.same_site = same_site;
		self
	}

	pub fn max_age(mut self, seconds: u64) -> Self {
		self.max_age = Some(seconds);
		self
	}

	pub fn path(mut self, path: impl Into<String>) -> Self {
		self.path = Some(path.into());
		self
	}

	pub fn domain(mut self, domain: impl Into<String>) -> Self {
		self.domain = Some(domain.into());
		self
	}

	pub fn to_header_value(&self) -> String {
		let mut parts = vec![format!("{}={}", self.name, self.value)];
		if let Some(p) = &self.path {
			parts.push(format!("Path={}", p));
		}
		if let Some(d) = &self.domain {
			parts.push(format!("Domain={}", d));
		}
		if let Some(age) = self.max_age {
			parts.push(format!("Max-Age={}", age));
		}
		if self.secure {
			parts.push("Secure".to_string());
		}
		if self.http_only {
			parts.push("HttpOnly".to_string());
		}
		parts.push(format!("SameSite={}", self.same_site.as_str()));
		parts.join("; ")
	}
}

// ─── MurRes ──────────────────────────────────────────────────────────────────

pub struct MurRes(pub(crate) RawRes);

fn json_body<T: Serialize>(status: StatusCode, body: T) -> RawRes {
	let encoded = serde_json::to_string(&body)
		.unwrap_or_else(|e| format!(r#"{{"error":"serialization failed: {}"}}"#, e));
	Ok(Response::builder()
		.status(status)
		.header("Content-Type", "application/json")
		.body(Full::new(Bytes::from(encoded)))
		.unwrap())
}

fn empty_body(status: StatusCode) -> RawRes {
	Ok(Response::builder()
		.status(status)
		.body(Full::new(Bytes::new()))
		.unwrap())
}

fn redirect_body(status: StatusCode, url: impl Into<String>) -> RawRes {
	Ok(Response::builder()
		.status(status)
		.header("Location", url.into())
		.body(Full::new(Bytes::new()))
		.unwrap())
}

impl MurRes {
	// ── Success ──────────────────────────────────────────────────────────────

	/// `200 OK` with `body` serialized as JSON.
	pub fn ok<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::OK, body))
	}

	/// `201 Created` with `body` serialized as JSON.
	pub fn created<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::CREATED, body))
	}

	/// `202 Accepted` with `body` serialized as JSON.
	pub fn accepted<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::ACCEPTED, body))
	}

	/// `204 No Content`.
	pub fn no_content() -> Self {
		Self(empty_body(StatusCode::NO_CONTENT))
	}

	/// Alias for [`ok`](Self::ok) — explicit JSON response.
	pub fn json<T: Serialize>(body: T) -> Self {
		Self::ok(body)
	}

	/// `200 OK` with plain-text body.
	pub fn text(body: impl Into<String>) -> Self {
		let b = body.into();
		Self(Ok(Response::builder()
			.status(StatusCode::OK)
			.header("Content-Type", "text/plain; charset=utf-8")
			.body(Full::new(Bytes::from(b)))
			.unwrap()))
	}

	/// `200 OK` with HTML body.
	pub fn html(body: impl Into<String>) -> Self {
		let b = body.into();
		Self(Ok(Response::builder()
			.status(StatusCode::OK)
			.header("Content-Type", "text/html; charset=utf-8")
			.body(Full::new(Bytes::from(b)))
			.unwrap()))
	}

	/// `200 OK` with raw byte body.
	pub fn bytes(body: impl Into<Bytes>) -> Self {
		Self(Ok(Response::builder()
			.status(StatusCode::OK)
			.header("Content-Type", "application/octet-stream")
			.body(Full::new(body.into()))
			.unwrap()))
	}

	// ── Redirects ────────────────────────────────────────────────────────────

	/// `302 Found` redirect.
	pub fn redirect(url: impl Into<String>) -> Self {
		Self(redirect_body(StatusCode::FOUND, url))
	}

	/// `301 Moved Permanently` redirect.
	pub fn permanent_redirect(url: impl Into<String>) -> Self {
		Self(redirect_body(StatusCode::MOVED_PERMANENTLY, url))
	}

	/// `303 See Other` redirect.
	pub fn see_other(url: impl Into<String>) -> Self {
		Self(redirect_body(StatusCode::SEE_OTHER, url))
	}

	/// `307 Temporary Redirect` (method preserved).
	pub fn temporary_redirect(url: impl Into<String>) -> Self {
		Self(redirect_body(StatusCode::TEMPORARY_REDIRECT, url))
	}

	/// `308 Permanent Redirect` (method preserved).
	pub fn permanent_redirect_keep_method(url: impl Into<String>) -> Self {
		Self(redirect_body(StatusCode::PERMANENT_REDIRECT, url))
	}

	// ── Client errors ────────────────────────────────────────────────────────

	/// `400 Bad Request`.
	pub fn bad_request<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::BAD_REQUEST, body))
	}

	/// `401 Unauthorized`.
	pub fn unauthorized<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::UNAUTHORIZED, body))
	}

	/// `403 Forbidden`.
	pub fn forbidden<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::FORBIDDEN, body))
	}

	/// `404 Not Found`.
	pub fn not_found<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::NOT_FOUND, body))
	}

	/// `409 Conflict`.
	pub fn conflict<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::CONFLICT, body))
	}

	/// `422 Unprocessable Entity`.
	pub fn unprocessable<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::UNPROCESSABLE_ENTITY, body))
	}

	/// `429 Too Many Requests`.
	pub fn too_many_requests<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::TOO_MANY_REQUESTS, body))
	}

	// ── Server errors ────────────────────────────────────────────────────────

	/// `500 Internal Server Error`.
	pub fn internal_error<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::INTERNAL_SERVER_ERROR, body))
	}

	/// `502 Bad Gateway`.
	pub fn bad_gateway<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::BAD_GATEWAY, body))
	}

	/// `503 Service Unavailable`.
	pub fn service_unavailable<T: Serialize>(body: T) -> Self {
		Self(json_body(StatusCode::SERVICE_UNAVAILABLE, body))
	}

	// ── Custom ───────────────────────────────────────────────────────────────

	/// Response with a custom status code and JSON body.
	pub fn status<T: Serialize>(code: StatusCode, body: T) -> Self {
		Self(json_body(code, body))
	}

	/// Response wrapping a `MurError` directly.
	pub fn err(e: impl Into<MurError>) -> Self {
		Self(Err(e.into()))
	}

	// ── Modifiers ────────────────────────────────────────────────────────────

	/// Appends a single response header. No-op on error responses.
	pub fn with_header(self, name: impl Into<String>, value: impl Into<String>) -> Self {
		self.map_response(|mut r| {
			if let (Ok(n), Ok(v)) = (
				name.into().parse::<http::header::HeaderName>(),
				value.into().parse::<http::header::HeaderValue>(),
			) {
				r.headers_mut().insert(n, v);
			}
			r
		})
	}

	/// Appends multiple response headers. No-op on error responses.
	pub fn with_headers(
		self,
		headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
	) -> Self {
		let pairs: Vec<(String, String)> = headers
			.into_iter()
			.map(|(k, v)| (k.into(), v.into()))
			.collect();
		self.map_response(move |mut r| {
			for (name, value) in &pairs {
				if let (Ok(n), Ok(v)) = (
					name.parse::<http::header::HeaderName>(),
					value.parse::<http::header::HeaderValue>(),
				) {
					r.headers_mut().insert(n, v);
				}
			}
			r
		})
	}

	/// Appends a `Set-Cookie` header. No-op on error responses.
	pub fn with_cookie(self, cookie: MurCookie) -> Self {
		let header_value = cookie.to_header_value();
		self.map_response(move |mut r| {
			if let Ok(v) = header_value.parse::<http::header::HeaderValue>() {
				r.headers_mut().append(http::header::SET_COOKIE, v);
			}
			r
		})
	}

	/// Overrides the HTTP status code. No-op on error responses.
	pub fn with_status(self, code: StatusCode) -> Self {
		self.map_response(move |mut r| {
			*r.status_mut() = code;
			r
		})
	}

	// ── Test / utility ───────────────────────────────────────────────────────

	pub fn is_ok(&self) -> bool {
		self.0.is_ok()
	}

	pub fn is_err(&self) -> bool {
		self.0.is_err()
	}

	pub fn unwrap(self) -> Response<Full<Bytes>> {
		self.0.unwrap()
	}

	pub fn unwrap_err(self) -> crate::server::error::MurError {
		self.0.unwrap_err()
	}

	// ── Internal ─────────────────────────────────────────────────────────────

	/// Applies `f` to the inner `Response` when `Ok`. Passes errors through unchanged.
	pub(crate) fn map_response(
		self,
		f: impl FnOnce(Response<Full<Bytes>>) -> Response<Full<Bytes>>,
	) -> Self {
		Self(self.0.map(f))
	}

	pub(crate) fn into_result(self) -> RawRes {
		self.0
	}

	pub(crate) fn from_result(r: RawRes) -> Self {
		Self(r)
	}
}

// ── From conversions ─────────────────────────────────────────────────────────

impl From<MurError> for MurRes {
	fn from(e: MurError) -> Self {
		Self(Err(e))
	}
}

impl From<RawRes> for MurRes {
	fn from(r: RawRes) -> Self {
		Self(r)
	}
}

impl From<Response<Full<Bytes>>> for MurRes {
	fn from(r: Response<Full<Bytes>>) -> Self {
		Self(Ok(r))
	}
}
