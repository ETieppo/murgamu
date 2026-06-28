//! End-to-end integration tests for the Murgamü framework.
//!
//! Unlike the unit tests scattered through `src/`, these tests boot a **real**
//! `MurServer` bound to a TCP port and exercise it through a real HTTP/1.1
//! client (built on hyper). They cover the concerns a backend framework must
//! get right:
//!
//! Functionality
//!   * routing (verbs, path params, query params, wildcards, 404, OPTIONS, HEAD)
//!   * request bodies (JSON, form-urlencoded) and malformed-input handling
//!   * response kinds (JSON, text, HTML) and content types
//!   * dependency injection (nested services, modules/providers)
//!   * interceptors (before short-circuit, after transform)
//!   * middleware (header injection, short-circuit)
//!   * the unified error type -> HTTP status mapping
//!
//! Security
//!   * authentication guards (401 vs 403, `#[public]` bypass)
//!   * role-based authorization (`#[role(..)]`)
//!   * request body size limits (413)
//!   * rate limiting / throttling (429 + headers)
//!   * CORS (permissive, restricted origins, rejected origins, preflight)
//!   * internal-error message leakage
//!
//! Each `#[tokio::test]` spins its own server on an ephemeral port so the suite
//! runs in parallel without interfering.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use http_body_util::{BodyExt, Full};
use hyper::Request;
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use murgamu::{MurServer, MurServerConfig, MurServerRunner, MurThrottler};
use tokio::net::TcpStream;

// ===========================================================================
// Test application: services, controllers, modules, guard, interceptor, mw
// ===========================================================================

mod app {
	use murgamu::prelude::*;
	use murgamu::{MurCookie, SameSite};
	use std::future::Future;
	use std::pin::Pin;

	// ---- services (dependency injection) ---------------------------------

	#[injectable]
	pub struct GreeterService;

	#[allow(dead_code)] // constructed by the DI factory via field-init
	impl GreeterService {
		pub fn new() -> Self {
			Self
		}

		pub fn greet(&self, who: &str) -> String {
			format!("Hello, {who}!")
		}
	}

	/// Depends on `GreeterService` to prove nested DI resolution works.
	#[injectable]
	pub struct UserService {
		greeter: GreeterService,
	}

	#[allow(dead_code)] // constructed by the DI factory via field-init
	impl UserService {
		pub fn new(greeter: GreeterService) -> Self {
			Self { greeter }
		}

		pub fn describe(&self, id: u32) -> String {
			format!("{} (user #{id})", self.greeter.greet("user"))
		}
	}

	// ---- DTOs ------------------------------------------------------------

	#[derive(Serialize, Deserialize, Debug)]
	pub struct EchoDto {
		pub name: String,
		pub value: i64,
	}

	#[derive(Deserialize, Default, Debug)]
	pub struct SearchQuery {
		pub term: Option<String>,
		pub limit: Option<u32>,
	}

	// ---- main API controller --------------------------------------------

	#[derive(Clone)]
	pub struct ApiController {
		users: UserService,
	}

	#[controller("/api")]
	impl ApiController {
		pub fn new(users: UserService) -> Self {
			Self { users }
		}

		#[get("/hello")]
		async fn hello(&self) -> MurRes {
			mur_json!({ "message": self.users.describe(0) })
		}

		#[get("/users/:id")]
		async fn get_user(&self, #[param] id: u32) -> MurRes {
			mur_json!({ "id": id, "describe": self.users.describe(id) })
		}

		#[get("/search")]
		async fn search(&self, query: MurQuery<SearchQuery>) -> MurRes {
			let term = query.term.clone().unwrap_or_else(|| "none".into());
			let limit = query.limit.unwrap_or(10);
			mur_json!({ "term": term, "limit": limit })
		}

		#[post("/echo")]
		async fn echo(&self, #[body] dto: EchoDto) -> MurRes {
			mur_json!({ "name": dto.name, "value": dto.value })
		}

		#[post("/form")]
		async fn form_ep(&self, ctx: MurRequestContext) -> MurRes {
			match ctx.form::<EchoDto>() {
				Ok(dto) => mur_json!({ "name": dto.name, "value": dto.value }),
				Err(e) => e.into(),
			}
		}

		#[put("/items/:id")]
		async fn update_item(&self, #[param] id: u32) -> MurRes {
			mur_json!({ "updated": id })
		}

		#[patch("/items/:id")]
		async fn patch_item(&self, #[param] id: u32) -> MurRes {
			mur_json!({ "patched": id })
		}

		#[delete("/items/:id")]
		async fn delete_item(&self, #[param] id: u32) -> MurRes {
			mur_json!({ "deleted": id })
		}

		#[get("/text")]
		async fn text_ep(&self) -> MurRes {
			mur_text!("plain text body")
		}

		#[get("/html")]
		async fn html_ep(&self) -> MurRes {
			mur_html!("<h1>hi</h1>")
		}

		// --- routing edge cases --------------------------------------------

		#[get("/files/*")]
		async fn wildcard_ep(&self) -> MurRes {
			mur_json!({ "matched": "wildcard" })
		}

		#[get("/tree/*rest")]
		async fn catchall_ep(&self, ctx: MurRequestContext) -> MurRes {
			mur_json!({ "rest": ctx.path_param("rest").unwrap_or("") })
		}

		// `/api/profile` (static) must win over `/api/:slug` (dynamic).
		#[get("/profile")]
		async fn static_profile(&self) -> MurRes {
			mur_json!({ "route": "static" })
		}

		#[get("/:slug")]
		async fn dynamic_slug(&self, #[param] slug: String) -> MurRes {
			mur_json!({ "route": "dynamic", "slug": slug })
		}

		// --- slow / large (timeout + compression) ---------------------------

		#[get("/slow")]
		async fn slow_ep(&self) -> MurRes {
			tokio::time::sleep(std::time::Duration::from_millis(400)).await;
			mur_json!({ "slow": true })
		}

		#[get("/large")]
		async fn large_ep(&self) -> MurRes {
			mur_json!({ "data": "a".repeat(4096) })
		}

		// --- cookies --------------------------------------------------------

		#[get("/cookie")]
		async fn set_cookie_ep(&self) -> MurRes {
			mur_json!({ "ok": true }).with_cookie(
				MurCookie::new("sid", "secret123")
					.http_only()
					.secure()
					.same_site(SameSite::Strict),
			)
		}

		// --- error mapping --------------------------------------------------

		#[get("/conflict")]
		async fn conflict_ep(&self) -> MurRes {
			MurError::conflict("already exists").into()
		}

		#[get("/teapot")]
		async fn teapot_ep(&self) -> MurRes {
			MurError::custom(StatusCode::IM_A_TEAPOT, "i am a teapot").into()
		}

		/// Returns an Internal error carrying a sensitive detail; the framework
		/// must NOT leak that detail to the client.
		#[get("/boom")]
		async fn boom_ep(&self) -> MurRes {
			MurError::internal("DB password is hunter2").into()
		}
	}

	#[module(
		controllers: [ApiController],
		providers: [GreeterService, UserService],
	)]
	pub struct AppModule;

	// ---- security: auth guard + protected controller ---------------------

	const TOKEN_ADMIN: &str = "admin-token";
	const TOKEN_USER: &str = "user-token";

	#[guard]
	pub struct AuthGuard;

	#[allow(dead_code)] // constructed by the guard factory via field-init
	impl AuthGuard {
		pub fn new() -> Self {
			Self
		}

		pub async fn can_activate(&self, ctx: &MurRequestContext) -> bool {
			// `#[public]` routes are always allowed.
			if ctx.access_control.is_public {
				return true;
			}

			// Require a recognised bearer token.
			let role = match ctx.bearer_token() {
				Some(t) if t == TOKEN_ADMIN => "admin",
				Some(t) if t == TOKEN_USER => "user",
				_ => return false, // missing / unknown token
			};

			// If the route declares roles, enforce them; otherwise any
			// authenticated caller is allowed.
			let allowed = ctx.allowed_roles();
			if allowed.is_empty() {
				return true;
			}
			allowed.contains(role)
		}
	}

	#[derive(Clone)]
	pub struct SecureController;

	#[controller("/secure")]
	impl SecureController {
		pub fn new() -> Self {
			Self
		}

		#[public]
		#[get("/open")]
		async fn open(&self) -> MurRes {
			mur_json!({ "scope": "public" })
		}

		#[get("/me")]
		async fn me(&self) -> MurRes {
			mur_json!({ "scope": "authenticated" })
		}

		#[role(admin)]
		#[get("/admin")]
		async fn admin(&self) -> MurRes {
			mur_json!({ "scope": "admin" })
		}
	}

	#[module(controllers: [SecureController])]
	pub struct SecureModule;

	// ---- interceptor (uses #[interceptor] macro + custom before/after) ------

	#[interceptor]
	pub struct StampInterceptor;

	impl MurInterceptor for StampInterceptor {
		fn before(&self, ctx: &MurRequestContext) -> MurInterceptorFuture {
			let deny = ctx.has_header("x-deny");
			Box::pin(async move {
				if deny {
					Err(MurError::unauthorized("denied by interceptor"))
				} else {
					Ok(())
				}
			})
		}

		fn after(
			&self,
			_ctx: &MurRequestContext,
			response: MurRes,
		) -> Pin<Box<dyn Future<Output = MurRes> + Send>> {
			Box::pin(async move { response.with_header("X-Intercepted", "true") })
		}

		fn name(&self) -> &str {
			"StampInterceptor"
		}
	}

	// ---- interceptor with DI (verifies #[interceptor] generates factory) -----

	/// Reads a value from an injected service and stamps it as a response header.
	#[interceptor]
	pub struct DependencyInterceptor {
		greeter: GreeterService,
	}

	impl MurInterceptor for DependencyInterceptor {
		fn after(
			&self,
			_ctx: &MurRequestContext,
			response: MurRes,
		) -> Pin<Box<dyn Future<Output = MurRes> + Send>> {
			let stamp = self.greeter.greet("interceptor");
			Box::pin(async move { response.with_header("X-DI-Interceptor", stamp) })
		}
	}

	// ---- no-op interceptor (verifies default trait methods still work) -------

	/// Default `before`/`after` do nothing; only the name is customized.
	#[interceptor]
	pub struct NoopInterceptor;

	impl MurInterceptor for NoopInterceptor {
		fn name(&self) -> &str {
			"NoopInterceptor"
		}
	}

	// ---- middleware (manual impl) ----------------------------------------

	#[derive(Clone)]
	pub struct StampMiddleware;

	impl MurMiddleware for StampMiddleware {
		fn handle(&self, ctx: MurRequestContext, next: MurNext) -> MurFuture {
			Box::pin(async move {
				if ctx.has_header("x-mw-block") {
					return MurHttpResponse::forbidden()
						.json(serde_json::json!({ "error": "blocked by middleware" }));
				}
				next.run(ctx).await.with_header("X-Middleware", "on")
			})
		}

		fn name(&self) -> &str {
			"StampMiddleware"
		}
	}
}

// ===========================================================================
// Test harness: ephemeral server + minimal HTTP client
// ===========================================================================

/// A running server bound to an ephemeral port. Dropping it signals shutdown.
struct TestServer {
	addr: SocketAddr,
	shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

impl TestServer {
	async fn start(runner: MurServerRunner) -> Self {
		let addr = runner.addr();
		let (tx, rx) = tokio::sync::oneshot::channel::<()>();

		tokio::spawn(async move {
			let _ = runner
				.run_until(async move {
					let _ = rx.await;
				})
				.await;
		});

		// Wait until the listener is accepting connections.
		for _ in 0..200 {
			if TcpStream::connect(addr).await.is_ok() {
				return Self {
					addr,
					shutdown: Some(tx),
				};
			}
			tokio::time::sleep(Duration::from_millis(10)).await;
		}
		panic!("server did not become ready on {addr}");
	}

	async fn send(
		&self,
		method: &str,
		path: &str,
		headers: &[(&str, &str)],
		body: Vec<u8>,
	) -> TestResponse {
		raw_request(self.addr, method, path, headers, body).await
	}

	async fn get(&self, path: &str) -> TestResponse {
		self.send("GET", path, &[], Vec::new()).await
	}

	async fn get_with(&self, path: &str, headers: &[(&str, &str)]) -> TestResponse {
		self.send("GET", path, headers, Vec::new()).await
	}

	async fn post_json(&self, path: &str, body: &str) -> TestResponse {
		self.send(
			"POST",
			path,
			&[("content-type", "application/json")],
			body.as_bytes().to_vec(),
		)
		.await
	}
}

impl Drop for TestServer {
	fn drop(&mut self) {
		if let Some(tx) = self.shutdown.take() {
			let _ = tx.send(());
		}
	}
}

struct TestResponse {
	status: u16,
	headers: HashMap<String, String>,
	body: Vec<u8>,
}

impl TestResponse {
	fn json(&self) -> serde_json::Value {
		serde_json::from_slice(&self.body).unwrap_or(serde_json::Value::Null)
	}

	fn text(&self) -> String {
		String::from_utf8_lossy(&self.body).to_string()
	}

	fn header(&self, name: &str) -> Option<&str> {
		self.headers.get(&name.to_lowercase()).map(|s| s.as_str())
	}
}

/// Performs a single HTTP/1.1 request over a fresh connection and collects the
/// full response.
async fn raw_request(
	addr: SocketAddr,
	method: &str,
	path: &str,
	headers: &[(&str, &str)],
	body: Vec<u8>,
) -> TestResponse {
	let stream = TcpStream::connect(addr).await.expect("connect");
	let io = TokioIo::new(stream);

	let (mut sender, conn): (hyper::client::conn::http1::SendRequest<Full<Bytes>>, _) =
		hyper::client::conn::http1::handshake(io)
			.await
			.expect("handshake");

	tokio::spawn(async move {
		let _ = conn.await;
	});

	let mut builder = Request::builder()
		.method(method)
		.uri(path)
		.header("Host", addr.to_string());
	for (k, v) in headers {
		builder = builder.header(*k, *v);
	}
	let req = builder
		.body(Full::new(Bytes::from(body)))
		.expect("build request");

	let res = sender.send_request(req).await.expect("send request");
	let status = res.status().as_u16();

	let mut hmap = HashMap::new();
	for (k, v) in res.headers() {
		hmap.insert(
			k.as_str().to_lowercase(),
			v.to_str().unwrap_or("").to_string(),
		);
	}

	let body = res
		.into_body()
		.collect()
		.await
		.expect("collect body")
		.to_bytes()
		.to_vec();

	TestResponse {
		status,
		headers: hmap,
		body,
	}
}

fn free_addr() -> SocketAddr {
	let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind ephemeral");
	let addr = listener.local_addr().expect("local_addr");
	drop(listener);
	addr
}

// ---- server flavours used by the tests ------------------------------------

/// Full-featured functional server: all routes public, interceptor + mw active.
async fn functional_server() -> TestServer {
	let addr = free_addr();
	let runner = MurServer::new()
		.no_logging()
		.default_public_routes()
		.global_interceptor(app::StampInterceptor)
		.middleware(app::StampMiddleware)
		.module(app::AppModule::new())
		.bind(addr)
		.expect("bind functional server");
	TestServer::start(runner).await
}

/// Server with a global DI-backed interceptor to verify the factory generated by `#[interceptor]`.
///
/// `DependencyInterceptor` injects `GreeterService` from the container.
/// `GreeterService` is registered directly (not via module) so it is visible
/// to global factories that run after module setup.
async fn di_interceptor_server() -> TestServer {
	let addr = free_addr();
	let runner = MurServer::new()
		.no_logging()
		.default_public_routes()
		.service(app::GreeterService::new())
		.global_interceptor(app::NoopInterceptor)
		.module(app::AppModule::new())
		.interceptor::<app::DependencyInterceptor>()
		.bind(addr)
		.expect("bind di-interceptor server");
	TestServer::start(runner).await
}

/// Server with a global authentication guard and protected routes.
async fn auth_server() -> TestServer {
	let addr = free_addr();
	let runner = MurServer::new()
		.no_logging()
		.guard::<app::AuthGuard>()
		.module(app::SecureModule::new())
		.bind(addr)
		.expect("bind auth server");
	TestServer::start(runner).await
}

// ===========================================================================
// Functional tests
// ===========================================================================

#[tokio::test]
async fn get_returns_json_body() {
	let server = functional_server().await;
	let res = server.get("/api/hello").await;

	assert_eq!(res.status, 200);
	assert_eq!(res.header("content-type"), Some("application/json"));
	assert!(res.json()["message"].as_str().unwrap().contains("Hello"));
}

#[tokio::test]
async fn path_param_is_parsed() {
	let server = functional_server().await;
	let res = server.get("/api/users/42").await;

	assert_eq!(res.status, 200);
	assert_eq!(res.json()["id"], 42);
}

#[tokio::test]
async fn invalid_path_param_is_rejected() {
	let server = functional_server().await;
	let res = server.get("/api/users/not-a-number").await;

	// u32 parse failure -> 400 Bad Request
	assert_eq!(res.status, 400);
}

#[tokio::test]
async fn query_params_are_parsed() {
	let server = functional_server().await;
	let res = server.get("/api/search?term=rust&limit=5").await;

	assert_eq!(res.status, 200);
	assert_eq!(res.json()["term"], "rust");
	assert_eq!(res.json()["limit"], 5);
}

#[tokio::test]
async fn query_params_use_defaults_when_missing() {
	let server = functional_server().await;
	let res = server.get("/api/search").await;

	assert_eq!(res.status, 200);
	assert_eq!(res.json()["term"], "none");
	assert_eq!(res.json()["limit"], 10);
}

#[tokio::test]
async fn post_json_body_is_deserialized() {
	let server = functional_server().await;
	let res = server
		.post_json("/api/echo", r#"{"name":"ada","value":7}"#)
		.await;

	assert_eq!(res.status, 200);
	assert_eq!(res.json()["name"], "ada");
	assert_eq!(res.json()["value"], 7);
}

#[tokio::test]
async fn malformed_json_body_is_rejected() {
	let server = functional_server().await;
	let res = server.post_json("/api/echo", "{ this is not json").await;

	assert_eq!(res.status, 400);
}

#[tokio::test]
async fn post_form_body_is_deserialized() {
	let server = functional_server().await;
	let res = server
		.send(
			"POST",
			"/api/form",
			&[("content-type", "application/x-www-form-urlencoded")],
			b"name=grace&value=99".to_vec(),
		)
		.await;

	assert_eq!(res.status, 200);
	assert_eq!(res.json()["name"], "grace");
	assert_eq!(res.json()["value"], 99);
}

#[tokio::test]
async fn put_patch_delete_reach_handlers() {
	let server = functional_server().await;

	let put = server.send("PUT", "/api/items/1", &[], Vec::new()).await;
	assert_eq!(put.status, 200);
	assert_eq!(put.json()["updated"], 1);

	let patch = server.send("PATCH", "/api/items/2", &[], Vec::new()).await;
	assert_eq!(patch.status, 200);
	assert_eq!(patch.json()["patched"], 2);

	let del = server.send("DELETE", "/api/items/3", &[], Vec::new()).await;
	assert_eq!(del.status, 200);
	assert_eq!(del.json()["deleted"], 3);
}

#[tokio::test]
async fn head_falls_back_to_get_handler() {
	let server = functional_server().await;
	let res = server.send("HEAD", "/api/hello", &[], Vec::new()).await;

	assert_eq!(res.status, 200);
}

#[tokio::test]
async fn options_lists_allowed_methods() {
	let server = functional_server().await;
	let res = server
		.send("OPTIONS", "/api/items/1", &[], Vec::new())
		.await;

	assert_eq!(res.status, 204);
	let allow = res.header("allow").unwrap_or_default();
	assert!(allow.contains("PUT"), "Allow header was: {allow}");
	assert!(allow.contains("DELETE"), "Allow header was: {allow}");
}

#[tokio::test]
async fn unknown_route_returns_404_json() {
	let server = functional_server().await;
	// Use a prefix with no registered controller so nothing (not even the
	// `/api/:slug` catch route) matches.
	let res = server.get("/totally/unknown/path").await;

	assert_eq!(res.status, 404);
	assert_eq!(res.json()["status"], 404);
}

#[tokio::test]
async fn text_and_html_content_types() {
	let server = functional_server().await;

	let text = server.get("/api/text").await;
	assert_eq!(text.status, 200);
	assert!(
		text.header("content-type")
			.unwrap_or_default()
			.contains("text/plain")
	);
	assert_eq!(text.text(), "plain text body");

	let html = server.get("/api/html").await;
	assert_eq!(html.status, 200);
	assert!(
		html.header("content-type")
			.unwrap_or_default()
			.contains("text/html")
	);
}

#[tokio::test]
async fn error_type_maps_to_status_codes() {
	let server = functional_server().await;

	let conflict = server.get("/api/conflict").await;
	assert_eq!(conflict.status, 409);

	let teapot = server.get("/api/teapot").await;
	assert_eq!(teapot.status, 418);
}

#[tokio::test]
async fn dependency_injection_resolves_nested_services() {
	// /api/hello exercises ApiController -> UserService -> GreeterService.
	let server = functional_server().await;
	let res = server.get("/api/users/5").await;

	assert_eq!(res.status, 200);
	let describe = res.json()["describe"].as_str().unwrap().to_string();
	assert!(describe.contains("user #5"), "got: {describe}");
}

#[tokio::test]
async fn interceptor_after_adds_header() {
	let server = functional_server().await;
	let res = server.get("/api/hello").await;

	assert_eq!(res.status, 200);
	assert_eq!(res.header("x-intercepted"), Some("true"));
}

#[tokio::test]
async fn interceptor_before_can_short_circuit() {
	let server = functional_server().await;
	let res = server.get_with("/api/hello", &[("x-deny", "1")]).await;

	assert_eq!(res.status, 401);
}

// --- #[interceptor] macro-specific tests ------------------------------------

/// Verifies that a unit-struct interceptor created with `#[interceptor]` and a
/// no-op `impl MurInterceptor` does not alter the response.
#[tokio::test]
async fn interceptor_macro_noop_does_not_affect_response() {
	let server = di_interceptor_server().await;
	let res = server.get("/api/hello").await;

	assert_eq!(res.status, 200);
}

/// Verifies that an interceptor created with `#[interceptor]` that injects a
/// service via DI resolves the dependency and stamps a response header.
#[tokio::test]
async fn interceptor_macro_with_di_stamps_header() {
	let server = di_interceptor_server().await;
	let res = server.get("/api/hello").await;

	assert_eq!(res.status, 200);
	let header = res.header("x-di-interceptor").unwrap_or_default();
	assert!(
		header.contains("interceptor"),
		"expected greeter output in X-DI-Interceptor, got: {header}"
	);
}

/// Verifies that a DI-backed interceptor can still short-circuit requests via
/// `before` (the supertrait bound is satisfied even without the macro generating
/// an `impl MurInterceptor` block).
#[tokio::test]
async fn interceptor_macro_noop_passes_requests_through() {
	let server = di_interceptor_server().await;

	let ok = server.get("/api/hello").await;
	assert_eq!(ok.status, 200);

	let not_found = server.get("/no-such-route").await;
	assert_eq!(not_found.status, 404);
}

#[tokio::test]
async fn middleware_adds_header() {
	let server = functional_server().await;
	let res = server.get("/api/hello").await;

	assert_eq!(res.header("x-middleware"), Some("on"));
}

#[tokio::test]
async fn middleware_can_short_circuit() {
	let server = functional_server().await;
	let res = server.get_with("/api/hello", &[("x-mw-block", "1")]).await;

	assert_eq!(res.status, 403);
	assert_eq!(res.json()["error"], "blocked by middleware");
}

// ===========================================================================
// Security tests
// ===========================================================================

#[tokio::test]
async fn public_route_bypasses_auth() {
	let server = auth_server().await;
	let res = server.get("/secure/open").await;

	assert_eq!(res.status, 200);
	assert_eq!(res.json()["scope"], "public");
}

#[tokio::test]
async fn protected_route_without_token_is_rejected() {
	let server = auth_server().await;
	let res = server.get("/secure/me").await;

	// Guard returns false -> default rejection is 403 Forbidden.
	assert_eq!(res.status, 403);
}

#[tokio::test]
async fn protected_route_with_invalid_token_is_rejected() {
	let server = auth_server().await;
	let res = server
		.get_with("/secure/me", &[("authorization", "Bearer nope")])
		.await;

	assert_eq!(res.status, 403);
}

#[tokio::test]
async fn protected_route_with_valid_token_is_allowed() {
	let server = auth_server().await;
	let res = server
		.get_with("/secure/me", &[("authorization", "Bearer user-token")])
		.await;

	assert_eq!(res.status, 200);
	assert_eq!(res.json()["scope"], "authenticated");
}

#[tokio::test]
async fn role_protected_route_denies_insufficient_role() {
	let server = auth_server().await;
	let res = server
		.get_with("/secure/admin", &[("authorization", "Bearer user-token")])
		.await;

	assert_eq!(res.status, 403);
}

#[tokio::test]
async fn role_protected_route_allows_matching_role() {
	let server = auth_server().await;
	let res = server
		.get_with("/secure/admin", &[("authorization", "Bearer admin-token")])
		.await;

	assert_eq!(res.status, 200);
	assert_eq!(res.json()["scope"], "admin");
}

#[tokio::test]
async fn body_size_limit_returns_413() {
	let addr = free_addr();
	let runner = MurServer::new()
		.default_public_routes()
		.configure(MurServerConfig::default().no_logging().body_size_limit(64))
		.module(app::AppModule::new())
		.bind(addr)
		.expect("bind body-limit server");
	let server = TestServer::start(runner).await;

	// Build a JSON body well over the 64-byte limit.
	let big_name = "x".repeat(512);
	let body = format!(r#"{{"name":"{big_name}","value":1}}"#);
	let res = server.post_json("/api/echo", &body).await;

	assert_eq!(res.status, 413);
}

#[tokio::test]
async fn body_within_limit_is_accepted() {
	let addr = free_addr();
	let runner = MurServer::new()
		.default_public_routes()
		.configure(
			MurServerConfig::default()
				.no_logging()
				.body_size_limit(4096),
		)
		.module(app::AppModule::new())
		.bind(addr)
		.expect("bind body-limit server");
	let server = TestServer::start(runner).await;

	let res = server
		.post_json("/api/echo", r#"{"name":"ok","value":1}"#)
		.await;
	assert_eq!(res.status, 200);
}

#[tokio::test]
async fn rate_limiter_blocks_after_quota() {
	let addr = free_addr();
	let runner = MurServer::new()
		.no_logging()
		.default_public_routes()
		.middleware(MurThrottler::new().global().requests(3).per_minutes(1))
		.module(app::AppModule::new())
		.bind(addr)
		.expect("bind rate-limit server");
	let server = TestServer::start(runner).await;

	// First 3 requests pass.
	for _ in 0..3 {
		let res = server.get("/api/hello").await;
		assert_eq!(res.status, 200);
	}

	// 4th exceeds the quota.
	let blocked = server.get("/api/hello").await;
	assert_eq!(blocked.status, 429);
	assert!(blocked.header("retry-after").is_some());
}

#[tokio::test]
async fn cors_permissive_allows_any_origin() {
	let server = functional_server().await;
	let res = server
		.get_with("/api/hello", &[("origin", "http://anywhere.test")])
		.await;

	assert_eq!(res.status, 200);
	assert_eq!(res.header("access-control-allow-origin"), Some("*"));
}

async fn cors_restricted_server() -> TestServer {
	let addr = free_addr();
	let runner = MurServer::new()
		.no_logging()
		.default_public_routes()
		.cors(vec!["http://allowed.test".to_string()])
		.module(app::AppModule::new())
		.bind(addr)
		.expect("bind cors server");
	TestServer::start(runner).await
}

#[tokio::test]
async fn cors_restricted_reflects_allowed_origin() {
	let server = cors_restricted_server().await;
	let res = server
		.get_with("/api/hello", &[("origin", "http://allowed.test")])
		.await;

	assert_eq!(res.status, 200);
	assert_eq!(
		res.header("access-control-allow-origin"),
		Some("http://allowed.test")
	);
}

#[tokio::test]
async fn cors_restricted_rejects_unknown_origin() {
	let server = cors_restricted_server().await;
	let res = server
		.get_with("/api/hello", &[("origin", "http://evil.test")])
		.await;

	assert_eq!(res.status, 403);
}

#[tokio::test]
async fn cors_preflight_returns_no_content_with_headers() {
	let server = cors_restricted_server().await;
	let res = server
		.send(
			"OPTIONS",
			"/api/hello",
			&[
				("origin", "http://allowed.test"),
				("access-control-request-method", "GET"),
			],
			Vec::new(),
		)
		.await;

	assert_eq!(res.status, 204);
	assert_eq!(
		res.header("access-control-allow-origin"),
		Some("http://allowed.test")
	);
	assert!(
		res.header("access-control-allow-methods")
			.unwrap_or_default()
			.contains("GET")
	);
}

#[tokio::test]
async fn internal_errors_do_not_leak_details() {
	let server = functional_server().await;
	let res = server.get("/api/boom").await;

	assert_eq!(res.status, 500);
	let body = res.text();
	assert!(
		!body.contains("hunter2"),
		"internal error detail leaked to client: {body}"
	);
	assert!(body.contains("Internal Server Error"));
}

// ===========================================================================
// Routing edge cases
// ===========================================================================

#[tokio::test]
async fn wildcard_matches_exactly_one_segment() {
	let server = functional_server().await;

	let ok = server.get("/api/files/doc.txt").await;
	assert_eq!(ok.status, 200);
	assert_eq!(ok.json()["matched"], "wildcard");

	// `*` consumes a single segment only; a deeper path must not match.
	let deep = server.get("/api/files/a/b").await;
	assert_eq!(deep.status, 404);
}

#[tokio::test]
async fn catch_all_captures_remaining_path() {
	let server = functional_server().await;
	let res = server.get("/api/tree/a/b/c").await;

	assert_eq!(res.status, 200);
	assert_eq!(res.json()["rest"], "a/b/c");
}

#[tokio::test]
async fn static_route_beats_dynamic_param() {
	let server = functional_server().await;

	let stat = server.get("/api/profile").await;
	assert_eq!(stat.status, 200);
	assert_eq!(stat.json()["route"], "static");

	let dynamic = server.get("/api/anything-else").await;
	assert_eq!(dynamic.status, 200);
	assert_eq!(dynamic.json()["route"], "dynamic");
	assert_eq!(dynamic.json()["slug"], "anything-else");
}

#[tokio::test]
async fn trailing_slash_is_normalized() {
	let server = functional_server().await;
	let res = server.get("/api/hello/").await;
	assert_eq!(res.status, 200);
}

#[tokio::test]
async fn wrong_method_on_existing_path_is_404() {
	let server = functional_server().await;
	// `/api/hello` is GET-only; POST has no matching route.
	let res = server.send("POST", "/api/hello", &[], Vec::new()).await;
	assert_eq!(res.status, 404);
}

#[tokio::test]
async fn empty_body_to_json_route_is_rejected() {
	let server = functional_server().await;
	let res = server.send("POST", "/api/echo", &[], Vec::new()).await;
	assert_eq!(res.status, 400);
}

#[tokio::test]
async fn json_content_type_with_charset_still_parses() {
	let server = functional_server().await;
	let res = server
		.send(
			"POST",
			"/api/echo",
			&[("content-type", "application/json; charset=utf-8")],
			br#"{"name":"x","value":1}"#.to_vec(),
		)
		.await;
	assert_eq!(res.status, 200);
	assert_eq!(res.json()["name"], "x");
}

// ===========================================================================
// Cookies
// ===========================================================================

#[tokio::test]
async fn set_cookie_carries_security_attributes() {
	let server = functional_server().await;
	let res = server.get("/api/cookie").await;

	assert_eq!(res.status, 200);
	let cookie = res.header("set-cookie").expect("Set-Cookie present");
	assert!(cookie.contains("sid=secret123"), "cookie: {cookie}");
	assert!(cookie.contains("HttpOnly"), "cookie: {cookie}");
	assert!(cookie.contains("Secure"), "cookie: {cookie}");
	assert!(cookie.contains("SameSite=Strict"), "cookie: {cookie}");
}

// ===========================================================================
// Security headers (opt-in middleware)
// ===========================================================================

async fn security_headers_server() -> TestServer {
	use murgamu::server::security::headers::{MurSecurityHeaders, XFrameOptions};
	let addr = free_addr();
	let runner = MurServer::new()
		.no_logging()
		.default_public_routes()
		.middleware(
			MurSecurityHeaders::new()
				.x_content_type_options(true)
				.x_frame_options(XFrameOptions::Deny),
		)
		.module(app::AppModule::new())
		.bind(addr)
		.expect("bind security-headers server");
	TestServer::start(runner).await
}

#[tokio::test]
async fn security_headers_are_applied_when_enabled() {
	let server = security_headers_server().await;
	let res = server.get("/api/hello").await;

	assert_eq!(res.status, 200);
	assert_eq!(res.header("x-content-type-options"), Some("nosniff"));
	assert_eq!(res.header("x-frame-options"), Some("DENY"));
}

#[tokio::test]
async fn security_headers_absent_by_default() {
	// Documents that the framework does NOT add security headers unless the
	// MurSecurityHeaders middleware is explicitly installed.
	let server = functional_server().await;
	let res = server.get("/api/hello").await;
	assert!(res.header("x-frame-options").is_none());
	assert!(res.header("x-content-type-options").is_none());
}

// ===========================================================================
// Rate limiting: keying & spoofability
// ===========================================================================

async fn ip_throttled_server() -> TestServer {
	let addr = free_addr();
	let runner = MurServer::new()
		.no_logging()
		.default_public_routes()
		.middleware(MurThrottler::new().by_ip().requests(2).per_minutes(1))
		.module(app::AppModule::new())
		.bind(addr)
		.expect("bind ip-throttled server");
	TestServer::start(runner).await
}

#[tokio::test]
async fn rate_limit_is_per_forwarded_ip() {
	let server = ip_throttled_server().await;
	let ip_a = [("x-forwarded-for", "10.0.0.1")];

	assert_eq!(server.get_with("/api/hello", &ip_a).await.status, 200);
	assert_eq!(server.get_with("/api/hello", &ip_a).await.status, 200);
	// Third request from the same IP exceeds the quota.
	assert_eq!(server.get_with("/api/hello", &ip_a).await.status, 429);
}

#[tokio::test]
async fn rate_limit_is_bypassable_by_spoofing_forwarded_ip() {
	// SECURITY NOTE: IP-based throttling trusts `X-Forwarded-For` verbatim, so
	// a client that rotates the header gets a fresh bucket each time. This test
	// documents that bypass — deployments must only trust XFF from a known proxy.
	let server = ip_throttled_server().await;

	for i in 0..6 {
		let spoofed = format!("10.0.0.{i}");
		let headers = [("x-forwarded-for", spoofed.as_str())];
		let res = server.get_with("/api/hello", &headers).await;
		assert_eq!(
			res.status, 200,
			"spoofed IP {spoofed} should get its own bucket"
		);
	}
}

// ===========================================================================
// Robustness: large/odd inputs must not crash the server
// ===========================================================================

#[tokio::test]
async fn very_long_query_string_does_not_crash() {
	let server = functional_server().await;
	let long = "a".repeat(8192);
	let res = server.get(&format!("/api/search?term={long}")).await;
	assert_eq!(res.status, 200);
	assert_eq!(res.json()["term"], long);
}

#[tokio::test]
async fn deeply_nested_catch_all_path_is_handled() {
	let server = functional_server().await;
	let deep = (0..200)
		.map(|i| i.to_string())
		.collect::<Vec<_>>()
		.join("/");
	let res = server.get(&format!("/api/tree/{deep}")).await;
	assert_eq!(res.status, 200);
}

#[tokio::test]
async fn many_distinct_routes_remain_independently_resolvable() {
	let server = functional_server().await;
	// Hit a spread of routes back-to-back; each must resolve to its own handler.
	assert_eq!(server.get("/api/text").await.text(), "plain text body");
	assert_eq!(server.get("/api/users/9").await.json()["id"], 9);
	assert_eq!(server.get("/api/profile").await.json()["route"], "static");
	assert_eq!(server.get("/api/conflict").await.status, 409);
}

// ===========================================================================
// Performance / load (generous ceilings to avoid CI flakiness)
// ===========================================================================

#[tokio::test]
async fn perf_handles_concurrent_requests() {
	let server = functional_server().await;
	let addr = server.addr;

	let start = std::time::Instant::now();
	let mut set = tokio::task::JoinSet::new();
	for _ in 0..200 {
		set.spawn(async move { raw_request(addr, "GET", "/api/hello", &[], Vec::new()).await });
	}

	let mut ok = 0;
	while let Some(res) = set.join_next().await {
		if res.expect("task panicked").status == 200 {
			ok += 1;
		}
	}
	let elapsed = start.elapsed();

	assert_eq!(ok, 200, "all concurrent requests should succeed");
	assert!(
		elapsed < Duration::from_secs(30),
		"200 concurrent requests took too long: {elapsed:?}"
	);
	eprintln!("perf: 200 concurrent requests in {elapsed:?}");
}

#[tokio::test]
async fn perf_sequential_throughput_is_reasonable() {
	let server = functional_server().await;

	let n = 300;
	let start = std::time::Instant::now();
	for _ in 0..n {
		let res = server.get("/api/hello").await;
		assert_eq!(res.status, 200);
	}
	let elapsed = start.elapsed();

	assert!(
		elapsed < Duration::from_secs(30),
		"{n} sequential requests took too long: {elapsed:?}"
	);
	let rps = n as f64 / elapsed.as_secs_f64();
	eprintln!("perf: {n} sequential requests in {elapsed:?} (~{rps:.0} req/s)");
}

// ===========================================================================
// Timeout middleware
// ===========================================================================

async fn timeout_server() -> TestServer {
	use murgamu::server::middleware::timeout::MurTimeout;
	let addr = free_addr();
	let runner = MurServer::new()
		.no_logging()
		.default_public_routes()
		.middleware(MurTimeout::from_millis(80))
		.module(app::AppModule::new())
		.bind(addr)
		.expect("bind timeout server");
	TestServer::start(runner).await
}

#[tokio::test]
async fn slow_handler_times_out() {
	let server = timeout_server().await;
	// /api/slow sleeps 400ms, well past the 80ms budget.
	let res = server.get("/api/slow").await;
	assert_eq!(res.status, 408);
}

#[tokio::test]
async fn fast_handler_under_timeout_succeeds() {
	let server = timeout_server().await;
	let res = server.get("/api/hello").await;
	assert_eq!(res.status, 200);
}

// ===========================================================================
// Compression middleware
// ===========================================================================

async fn compression_server() -> TestServer {
	use murgamu::server::middleware::compression::MurCompression;
	let addr = free_addr();
	let runner = MurServer::new()
		.no_logging()
		.default_public_routes()
		.middleware(MurCompression::gzip_only())
		.module(app::AppModule::new())
		.bind(addr)
		.expect("bind compression server");
	TestServer::start(runner).await
}

#[tokio::test]
async fn large_response_is_gzip_compressed_when_accepted() {
	let server = compression_server().await;
	let res = server
		.get_with("/api/large", &[("accept-encoding", "gzip")])
		.await;

	assert_eq!(res.status, 200);
	assert_eq!(res.header("content-encoding"), Some("gzip"));
	// Highly repetitive ~4KB JSON must shrink dramatically.
	assert!(
		res.body.len() < 4096,
		"compressed body len: {}",
		res.body.len()
	);

	// The payload must be a valid gzip stream that restores the original JSON.
	use std::io::Read;
	let mut decoder = flate2::read::GzDecoder::new(&res.body[..]);
	let mut restored = String::new();
	decoder.read_to_string(&mut restored).expect("valid gzip");
	let value: serde_json::Value = serde_json::from_str(&restored).expect("valid json");
	assert_eq!(value["data"].as_str().unwrap().len(), 4096);
}

#[tokio::test]
async fn response_not_compressed_without_accept_encoding() {
	let server = compression_server().await;
	let res = server.get("/api/large").await;

	assert_eq!(res.status, 200);
	assert!(res.header("content-encoding").is_none());
}

#[tokio::test]
async fn small_response_is_not_compressed() {
	let server = compression_server().await;
	// /api/hello is far below the 860-byte min_size threshold.
	let res = server
		.get_with("/api/hello", &[("accept-encoding", "gzip")])
		.await;

	assert_eq!(res.status, 200);
	assert!(res.header("content-encoding").is_none());
}
