use super::MurRouteInfo;
use super::entry::MurRouteEntry;
use super::pattern::MurRoutePattern;
use crate::server::aliases::{MurPathParams, MurRes, MurRouteHandler};
use crate::server::controller::MurController;
use crate::server::error::MurError;
use crate::server::error::MurExceptionFilter;
use crate::server::guard::MurGuard;
use crate::server::http::MurHttpResponse;
use crate::server::http::MurRequestContext;
use crate::server::interceptor::MurInterceptor;
use crate::server::middleware::MurMiddleware;
use crate::server::router::MurRouteAccessControl;
use crate::server::service::MurServiceContainer;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, StatusCode};
use std::collections::HashMap;
use std::sync::Arc;

pub struct MurRouter {
	pub(crate) routes_by_method: HashMap<String, Vec<MurRouteEntry>>,
	pub(crate) global_guards: Vec<Arc<dyn MurGuard>>,
	pub(crate) global_interceptors: Vec<Arc<dyn MurInterceptor>>,
	pub(crate) global_middleware: Vec<Arc<dyn MurMiddleware>>,
	pub(crate) exception_filters: Vec<Arc<dyn MurExceptionFilter>>,
	pub(crate) container: Arc<MurServiceContainer>,
	pub(crate) route_info: Vec<MurRouteInfo>,
	pub(crate) not_found_handler: Option<MurRouteHandler>,
	pub(crate) error_handler: Option<Arc<dyn Fn(MurError) -> MurRes + Send + Sync>>,
	pub(crate) registered_methods: Vec<String>,
}

impl MurRouter {
	pub fn new(container: Arc<MurServiceContainer>) -> Self {
		let mut routes_by_method = HashMap::with_capacity(8);
		for method in &["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"] {
			routes_by_method.insert(method.to_string(), Vec::new());
		}

		Self {
			routes_by_method,
			global_guards: Vec::new(),
			global_interceptors: Vec::new(),
			global_middleware: Vec::new(),
			exception_filters: Vec::new(),
			container,
			route_info: Vec::new(),
			not_found_handler: None,
			error_handler: None,
			registered_methods: Vec::new(),
		}
	}

	pub fn register_controller(&mut self, controller: Arc<dyn MurController>) {
		let controller_name = controller.name().to_string();
		let routes = controller.routes(Arc::clone(&self.container));

		for route_def in routes {
			let pattern = MurRoutePattern::new(&route_def.path);
			let mut entry = MurRouteEntry::new(pattern, route_def.handler);

			entry.access_control = MurRouteAccessControl {
				is_public: route_def.is_public,
				allowed_roles: route_def.allowed_roles.into_iter().collect(),
			};

			self.route_info.push(MurRouteInfo {
				method: route_def.method.clone(),
				path: route_def.path.clone(),
				controller: controller_name.clone(),
				handler: String::new(),
			});

			self.routes_by_method
				.entry(route_def.method.clone())
				.or_default()
				.push(entry);

			if !self.registered_methods.contains(&route_def.method) {
				self.registered_methods.push(route_def.method);
			}
		}

		self.sort_all_routes();
	}

	pub fn route(&mut self, method: &str, path: &str, handler: MurRouteHandler) {
		let method = method.to_uppercase();
		let pattern = MurRoutePattern::new(path);
		let entry = MurRouteEntry::new(pattern, handler);

		self.route_info.push(MurRouteInfo {
			method: method.clone(),
			path: path.to_string(),
			controller: String::from("manual"),
			handler: String::new(),
		});

		self.routes_by_method
			.entry(method.clone())
			.or_default()
			.push(entry);

		if !self.registered_methods.contains(&method) {
			self.registered_methods.push(method.clone());
		}

		if let Some(routes) = self.routes_by_method.get_mut(&method) {
			routes.sort_by(|a, b| {
				b.pattern
					.specificity_score()
					.cmp(&a.pattern.specificity_score())
			});
		}
	}

	#[inline]
	pub fn get(&mut self, path: &str, handler: MurRouteHandler) {
		self.route("GET", path, handler);
	}

	#[inline]
	pub fn post(&mut self, path: &str, handler: MurRouteHandler) {
		self.route("POST", path, handler);
	}

	#[inline]
	pub fn put(&mut self, path: &str, handler: MurRouteHandler) {
		self.route("PUT", path, handler);
	}

	#[inline]
	pub fn delete(&mut self, path: &str, handler: MurRouteHandler) {
		self.route("DELETE", path, handler);
	}

	#[inline]
	pub fn patch(&mut self, path: &str, handler: MurRouteHandler) {
		self.route("PATCH", path, handler);
	}

	pub fn add_guard(&mut self, guard: impl MurGuard + 'static) {
		self.global_guards.push(Arc::new(guard));
	}

	pub fn add_guard_boxed(&mut self, guard: Box<dyn MurGuard>) {
		self.global_guards.push(Arc::from(guard));
	}

	pub fn add_interceptor(&mut self, interceptor: impl MurInterceptor + 'static) {
		self.global_interceptors.push(Arc::new(interceptor));
	}

	pub fn add_interceptor_boxed(&mut self, interceptor: Box<dyn MurInterceptor>) {
		self.global_interceptors.push(Arc::from(interceptor));
	}

	pub fn add_middleware(&mut self, middleware: impl MurMiddleware + 'static) {
		self.global_middleware.push(Arc::new(middleware));
	}

	pub async fn execute_matched_route(
		&self,
		method: &str,
		path: &str,
		ctx: MurRequestContext,
	) -> Option<MurRes> {
		let method = method.to_uppercase();
		if let Some(routes) = self.routes_by_method.get(&method) {
			for route in routes {
				if route.pattern.match_path(path).is_some() {
					return Some((route.handler)(ctx).await);
				}
			}
		}
		None
	}

	pub fn find_route_params(&self, method: &str, path: &str) -> Option<MurPathParams> {
		let method = method.to_uppercase();
		if let Some(routes) = self.routes_by_method.get(&method) {
			for route in routes {
				if let Some(params) = route.pattern.match_path(path) {
					return Some(params);
				}
			}
		}
		None
	}

	pub fn add_exception_filter(&mut self, filter: impl MurExceptionFilter + 'static) {
		self.exception_filters.push(Arc::new(filter));
	}

	pub fn set_not_found_handler(&mut self, handler: MurRouteHandler) {
		self.not_found_handler = Some(handler);
	}

	pub fn set_error_handler(
		&mut self,
		handler: impl Fn(MurError) -> MurRes + Send + Sync + 'static,
	) {
		self.error_handler = Some(Arc::new(handler));
	}

	#[inline]
	pub fn route_info(&self) -> &[MurRouteInfo] {
		&self.route_info
	}

	pub fn print_routes(&self) {
		const THRESHOLD: usize = 22;
		const END_THRESHOLD: usize = 18;
		const START_ROUTES_AT: usize = 3;
		const PAPIRUS_SIDE: [&str; 8] = [
			"     ,d' ",
			"     8'  ",
			"     8,  ",
			"     `b, ",
			"      `b,",
			"      `8 ",
			"      8  ",
			"    ,8   ",
		];
		let mut routes = Vec::new();
		let mut max_width = 0;
		let mut idx = 0;

		for info in self.route_info() {
			let size = info.controller.len() + info.method.len() + info.path.len();
			if size > max_width {
				max_width = size;
			}
			routes.push((
				info.method.clone(),
				info.path.clone(),
				info.controller.clone(),
				size,
			));
		}

		println!(
			"     ,Yaaa{:a<w$}dbb,    ",
			"a",
			w = max_width - END_THRESHOLD
		);
		println!("   ,d\"{:w$}  a     d", "", w = max_width - END_THRESHOLD);
		println!(
			"   p\"     {:^w$} i      ,d ",
			"ROUTER",
			w = (max_width / 2) + END_THRESHOLD - 10
		);
		println!("    8l  {:w$}L,   ,d'  ", "", w = max_width - END_THRESHOLD);
		println!(
			"    `Yaaa{:a<w$}dbP\"    ",
			"a",
			w = max_width - END_THRESHOLD
		);

		for _ in 0..START_ROUTES_AT {
			println!(
				"{}{:w$}{}",
				PAPIRUS_SIDE[idx],
				"",
				PAPIRUS_SIDE[idx],
				w = max_width - THRESHOLD
			);
			idx = (idx + 1) % PAPIRUS_SIDE.len();
		}

		for (method, path, controller, size) in routes {
			let last = controller.split("::").last().unwrap_or("--");
			println!(
				"{}    - {} {} -> {} {:w$}{}",
				PAPIRUS_SIDE[idx],
				method,
				path,
				last,
				"",
				PAPIRUS_SIDE[idx],
				w = max_width - size
			);
			idx = (idx + 1) % PAPIRUS_SIDE.len();
		}

		while idx < PAPIRUS_SIDE.len() {
			println!(
				"{}{:w$}{}",
				PAPIRUS_SIDE[idx],
				"",
				PAPIRUS_SIDE[idx],
				w = max_width - THRESHOLD
			);
			idx += 1;
		}

		println!(
			" ,aadb{:a<w$}aaaaa,d'   8",
			"a",
			w = max_width - THRESHOLD - 4
		);
		println!(",d\"{:w$}a      d", "", w = max_width - END_THRESHOLD);
		println!("p\" {:w$}\"i      ,d ", "", w = max_width - END_THRESHOLD);
		println!(" 8l  {:w$}L,   ,d'  ", "", w = max_width - END_THRESHOLD);
		println!(" `Yaaa{:a<w$}dbP\"    ", "a", w = max_width - END_THRESHOLD);
		println!();
	}

	pub async fn handle(&self, req: Request<Incoming>) -> MurRes {
		let (parts, body) = req.into_parts();
		let method = parts.method.to_string().to_uppercase();
		let path = parts.uri.path();
		let route_match = self.find_route(&method, path);

		if route_match.is_none() {
			if method == "OPTIONS" {
				return self.handle_options(path);
			}

			if method == "HEAD"
				&& let Some((route, params)) = self.find_route("GET", path)
			{
				let body_bytes = self.collect_body(body).await?;
				let ctx =
					MurRequestContext::new(parts, body_bytes, params, Arc::clone(&self.container));
				return self.execute_handler(route, ctx).await;
			}

			return self.handle_not_found(path);
		}

		let (route, path_params) = route_match.unwrap();
		let body_bytes = self.collect_body(body).await?;
		let ctx =
			MurRequestContext::new(parts, body_bytes, path_params, Arc::clone(&self.container));

		self.execute_handler(route, ctx).await
	}

	#[inline]
	async fn collect_body(&self, body: Incoming) -> Result<Option<Bytes>, MurError> {
		match body.collect().await {
			Ok(collected) => {
				let bytes = collected.to_bytes();
				if bytes.is_empty() {
					Ok(None)
				} else {
					Ok(Some(bytes))
				}
			}
			Err(e) => Err(MurError::from(e)),
		}
	}

	async fn execute_handler(&self, route: &MurRouteEntry, ctx: MurRequestContext) -> MurRes {
		let ctx = ctx.with_access_control(route.access_control.clone());

		for guard in &self.global_guards {
			if !guard.can_activate(&ctx).await {
				return guard.rejection_response();
			}
		}

		for guard in &route.guards {
			if !guard.can_activate(&ctx).await {
				return guard.rejection_response();
			}
		}

		for interceptor in &self.global_interceptors {
			if let Err(e) = interceptor.before(&ctx).await {
				return self.handle_error(e);
			}
		}

		for interceptor in &route.interceptors {
			if let Err(e) = interceptor.before(&ctx).await {
				return self.handle_error(e);
			}
		}

		let result = (route.handler)(ctx.clone()).await;
		let mut response = result;

		for interceptor in route.interceptors.iter().rev() {
			response = interceptor.after(&ctx, response).await;
		}

		for interceptor in self.global_interceptors.iter().rev() {
			response = interceptor.after(&ctx, response).await;
		}

		match response {
			Ok(res) => Ok(res),
			Err(e) => self.handle_error(e),
		}
	}

	#[inline]
	fn find_route(&self, method: &str, path: &str) -> Option<(&MurRouteEntry, MurPathParams)> {
		let routes = self.routes_by_method.get(method)?;

		for route in routes {
			if let Some(params) = route.pattern.match_path(path) {
				return Some((route, params));
			}
		}
		None
	}

	pub(crate) fn sort_all_routes(&mut self) {
		for routes in self.routes_by_method.values_mut() {
			routes.sort_by(|a, b| {
				b.pattern
					.specificity_score()
					.cmp(&a.pattern.specificity_score())
			});
		}
	}

	fn handle_not_found(&self, path: &str) -> MurRes {
		if let Some(_handler) = &self.not_found_handler {
			let parts = http::Request::builder()
				.uri(path)
				.method("GET")
				.body(())
				.unwrap()
				.into_parts()
				.0;
			let _ctx = MurRequestContext::new(
				parts,
				None,
				MurPathParams::new(),
				Arc::clone(&self.container),
			);
		}

		MurHttpResponse::not_found().json(serde_json::json!({
			"error": "Not Found",
			"message": format!("No route found for path: {}", path),
			"status": 404
		}))
	}

	fn handle_options(&self, path: &str) -> MurRes {
		let mut methods = Vec::with_capacity(8);

		for (method, routes) in &self.routes_by_method {
			for route in routes {
				if route.pattern.match_path(path).is_some() {
					methods.push(method.as_str());
					break;
				}
			}
		}

		let allow = if methods.is_empty() {
			"GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD".to_string()
		} else {
			methods.join(", ")
		};

		Ok(Response::builder()
			.status(StatusCode::NO_CONTENT)
			.header("Allow", &allow)
			.header("Access-Control-Allow-Methods", &allow)
			.header(
				"Access-Control-Allow-Headers",
				"Content-Type, Authorization",
			)
			.header("Access-Control-Max-Age", "86400")
			.body(Full::new(Bytes::new()))
			.unwrap())
	}

	fn handle_error(&self, error: MurError) -> MurRes {
		for filter in &self.exception_filters {
			if filter.can_handle(&error) {
				let parts = http::Request::builder()
					.uri("/")
					.method("GET")
					.body(())
					.unwrap()
					.into_parts()
					.0;
				let ctx = MurRequestContext::new(
					parts,
					None,
					MurPathParams::new(),
					Arc::clone(&self.container),
				);
				return filter.catch(error, &ctx);
			}
		}

		if let Some(handler) = &self.error_handler {
			return handler(error);
		}

		error.into_response()
	}
}
