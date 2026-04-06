use super::AllowedHeaders;
use super::AllowedOrigins;
// TODO: this file is too large
// FIX:
use super::DEFAULT_METHODS;
use super::MurCorsConfig;
use crate::server::aliases::MurFuture;
use crate::server::http::MurHttpResponse;
use crate::server::http::MurRequestContext;
use crate::server::middleware::{MurMiddleware, MurNext};
use http::Method;
use http::header::{
	ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
	ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_EXPOSE_HEADERS, ACCESS_CONTROL_MAX_AGE,
	ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, HeaderName, HeaderValue, ORIGIN,
	VARY,
};
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone)]
pub struct MurCors {
	pub config: Arc<MurCorsConfig>,
}

impl MurCors {
	pub fn new() -> Self {
		Self {
			config: Arc::new(MurCorsConfig::default()),
		}
	}

	pub fn permissive() -> Self {
		Self {
			config: Arc::new(MurCorsConfig {
				allowed_origins: AllowedOrigins::Any,
				allowed_methods: Cow::Borrowed(DEFAULT_METHODS),
				allowed_headers: AllowedHeaders::Any,
				exposed_headers: HashSet::new(),
				allow_credentials: false,
				max_age: Some(86400),
				allow_private_network: true,
				send_vary: true,
			}),
		}
	}

	pub fn strict() -> Self {
		Self {
			config: Arc::new(MurCorsConfig {
				allowed_origins: AllowedOrigins::List(HashSet::new()),
				allowed_methods: Cow::Borrowed(&[]),
				allowed_headers: AllowedHeaders::List(HashSet::new()),
				exposed_headers: HashSet::new(),
				allow_credentials: false,
				max_age: None,
				allow_private_network: false,
				send_vary: true,
			}),
		}
	}

	pub fn from_config(config: MurCorsConfig) -> Self {
		Self {
			config: Arc::new(config),
		}
	}

	pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
		let config = Arc::make_mut(&mut self.config);
		match &mut config.allowed_origins {
			AllowedOrigins::List(origins) => {
				origins.insert(origin.into());
			}
			_ => {
				let mut origins = HashSet::new();
				origins.insert(origin.into());
				config.allowed_origins = AllowedOrigins::List(origins);
			}
		}
		self
	}

	pub fn allow_origins<I, S>(mut self, origins: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		let config = Arc::make_mut(&mut self.config);
		let origins_set: HashSet<String> = origins.into_iter().map(|s| s.into()).collect();
		config.allowed_origins = AllowedOrigins::List(origins_set);
		self
	}

	pub fn allow_any_origin(mut self) -> Self {
		let config = Arc::make_mut(&mut self.config);
		config.allowed_origins = AllowedOrigins::Any;
		self
	}

	pub fn mirror_origin(mut self) -> Self {
		let config = Arc::make_mut(&mut self.config);
		config.allowed_origins = AllowedOrigins::Mirror;
		self
	}

	pub fn allow_methods<I, S>(mut self, methods: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: AsRef<str>,
	{
		let config = Arc::make_mut(&mut self.config);
		config.allowed_methods = methods
			.into_iter()
			.filter_map(|m| m.as_ref().parse().ok())
			.collect();
		self
	}

	pub fn allow_method(mut self, method: impl AsRef<str>) -> Self {
		let config = Arc::make_mut(&mut self.config);
		if let Ok(m) = method.as_ref().parse() {
			config.allowed_methods.to_mut().push(m);
		}
		self
	}

	pub fn allow_headers<I, S>(mut self, headers: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		let config = Arc::make_mut(&mut self.config);
		let headers_set: HashSet<String> = headers
			.into_iter()
			.map(|s| s.into().to_lowercase())
			.collect();
		config.allowed_headers = AllowedHeaders::List(headers_set);
		self
	}

	pub fn allow_any_header(mut self) -> Self {
		let config = Arc::make_mut(&mut self.config);
		config.allowed_headers = AllowedHeaders::Any;
		self
	}

	pub fn mirror_headers(mut self) -> Self {
		let config = Arc::make_mut(&mut self.config);
		config.allowed_headers = AllowedHeaders::Mirror;
		self
	}

	pub fn expose_headers<I, S>(mut self, headers: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		let config = Arc::make_mut(&mut self.config);
		config.exposed_headers = headers.into_iter().map(|s| s.into()).collect();
		self
	}

	pub fn allow_credentials(mut self, allow: bool) -> Self {
		let config = Arc::make_mut(&mut self.config);
		config.allow_credentials = allow;
		if allow && matches!(config.allowed_origins, AllowedOrigins::Any) {
			config.allowed_origins = AllowedOrigins::Mirror;
		}

		self
	}

	pub fn max_age(mut self, seconds: u64) -> Self {
		let config = Arc::make_mut(&mut self.config);
		config.max_age = Some(seconds);
		self
	}

	pub fn no_max_age(mut self) -> Self {
		let config = Arc::make_mut(&mut self.config);
		config.max_age = None;
		self
	}

	pub fn allow_private_network(mut self, allow: bool) -> Self {
		let config = Arc::make_mut(&mut self.config);
		config.allow_private_network = allow;
		self
	}

	pub fn send_vary(mut self, send: bool) -> Self {
		let config = Arc::make_mut(&mut self.config);
		config.send_vary = send;
		self
	}

	fn is_preflight(&self, ctx: &MurRequestContext) -> bool {
		ctx.method() == Method::OPTIONS
			&& ctx.header(ACCESS_CONTROL_REQUEST_METHOD.as_str()).is_some()
	}

	fn is_origin_allowed(&self, origin: &str) -> bool {
		self.config.allowed_origins.is_allowed(origin)
	}

	fn is_method_allowed(&self, method: &Method) -> bool {
		self.config.allowed_methods.contains(method)
	}

	fn build_cors_headers(
		&self,
		origin: Option<&str>,
		requested_headers: Option<&str>,
		is_preflight: bool,
	) -> Vec<(HeaderName, HeaderValue)> {
		let mut headers = Vec::new();

		if let Some(allow_origin) = self.config.allowed_origins.header_value(origin)
			&& let Ok(value) = HeaderValue::from_str(&allow_origin)
		{
			headers.push((ACCESS_CONTROL_ALLOW_ORIGIN, value));
		}

		if self.config.allow_credentials {
			headers.push((
				ACCESS_CONTROL_ALLOW_CREDENTIALS,
				HeaderValue::from_static("true"),
			));
		}

		if is_preflight {
			let methods: Vec<String> = self
				.config
				.allowed_methods
				.iter()
				.map(|m| m.to_string())
				.collect();
			if !methods.is_empty()
				&& let Ok(value) = HeaderValue::from_str(&methods.join(", "))
			{
				headers.push((ACCESS_CONTROL_ALLOW_METHODS, value));
			}

			if let Some(allow_headers) = self.config.allowed_headers.header_value(requested_headers)
				&& let Ok(value) = HeaderValue::from_str(&allow_headers)
			{
				headers.push((ACCESS_CONTROL_ALLOW_HEADERS, value));
			}

			if let Some(max_age) = self.config.max_age
				&& let Ok(value) = HeaderValue::from_str(&max_age.to_string())
			{
				headers.push((ACCESS_CONTROL_MAX_AGE, value));
			}

			if self.config.allow_private_network {
				let name = HeaderName::from_static("access-control-allow-private-network");
				headers.push((name, HeaderValue::from_static("true")));
			}
		} else if !self.config.exposed_headers.is_empty() {
			let exposed: Vec<&str> = self
				.config
				.exposed_headers
				.iter()
				.map(|s| s.as_str())
				.collect();
			if let Ok(value) = HeaderValue::from_str(&exposed.join(", ")) {
				headers.push((ACCESS_CONTROL_EXPOSE_HEADERS, value));
			}
		}

		if self.config.send_vary {
			headers.push((
				VARY,
				HeaderValue::from_static(
					"Origin, Access-Control-Request-Method, Access-Control-Request-Headers",
				),
			));
		}

		headers
	}
}

impl Default for MurCors {
	fn default() -> Self {
		Self::new()
	}
}

impl std::fmt::Debug for MurCors {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurCors")
			.field("config", &self.config)
			.finish()
	}
}

impl MurMiddleware for MurCors {
	fn handle(&self, ctx: MurRequestContext, next: MurNext) -> MurFuture {
		let _config = Arc::clone(&self.config);
		let cors = self.clone();

		Box::pin(async move {
			let Some(origin) = ctx.header(ORIGIN.as_str()) else {
				return next.run(ctx).await;
			};

			if !cors.is_origin_allowed(origin) {
				return MurHttpResponse::forbidden().json(serde_json::json!({
					"error": "CORS",
					"message": format!("Origin '{}' is not allowed", origin)
				}));
			}

			if cors.is_preflight(&ctx) {
				if let Some(requested_method) = ctx.header(ACCESS_CONTROL_REQUEST_METHOD.as_str())
					&& let Ok(method) = requested_method.parse::<Method>()
					&& !cors.is_method_allowed(&method)
				{
					return MurHttpResponse::forbidden().json(serde_json::json!({
						"error": "CORS",
						"message": format!("Method '{}' is not allowed", method)
					}));
				}

				let requested_headers = ctx
					.header(ACCESS_CONTROL_REQUEST_HEADERS.as_str())
					.map(|s| s.to_string());
				let headers =
					cors.build_cors_headers(Some(origin), requested_headers.as_deref(), true);

				let response = MurHttpResponse::no_content();
				if let Ok(mut resp) = response {
					for (name, value) in headers {
						resp.headers_mut().insert(name, value);
					}
					return Ok(resp);
				}
				return response;
			}

			let requested_headers = ctx
				.header(ACCESS_CONTROL_REQUEST_HEADERS.as_str())
				.map(|s| s.to_string());
			let cors_headers =
				cors.build_cors_headers(Some(origin), requested_headers.as_deref(), false);
			let result = next.run(ctx).await;

			match result {
				Ok(mut response) => {
					for (name, value) in cors_headers {
						response.headers_mut().insert(name, value);
					}
					Ok(response)
				}
				Err(e) => Err(e),
			}
		})
	}

	fn name(&self) -> &str {
		"MurCors"
	}
}

pub fn mur_cors_origins<I, S>(origins: I) -> MurCors
where
	I: IntoIterator<Item = S>,
	S: Into<String>,
{
	MurCors::new().allow_origins(origins)
}
