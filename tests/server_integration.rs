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
use hyper::body::Bytes;
use hyper::Request;
use hyper_util::rt::TokioIo;
use murgamu::{MurServer, MurServerConfig, MurServerRunner, MurThrottler};
use tokio::net::TcpStream;

// ===========================================================================
// Test application: services, controllers, modules, guard, interceptor, mw
// ===========================================================================

mod app {
	use murgamu::prelude::*;
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

	// ---- interceptor (manual impl for custom before/after) ---------------

	#[derive(Clone)]
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

	let (mut sender, conn): (
		hyper::client::conn::http1::SendRequest<Full<Bytes>>,
		_,
	) = hyper::client::conn::http1::handshake(io)
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
		.add_global_interceptor(app::StampInterceptor)
		.add_middleware(app::StampMiddleware)
		.module(app::AppModule::new())
		.bind(addr)
		.expect("bind functional server");
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
	assert_eq!(
		res.header("content-type"),
		Some("application/json")
	);
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
	let res = server.send("OPTIONS", "/api/items/1", &[], Vec::new()).await;

	assert_eq!(res.status, 204);
	let allow = res.header("allow").unwrap_or_default();
	assert!(allow.contains("PUT"), "Allow header was: {allow}");
	assert!(allow.contains("DELETE"), "Allow header was: {allow}");
}

#[tokio::test]
async fn unknown_route_returns_404_json() {
	let server = functional_server().await;
	let res = server.get("/api/does-not-exist").await;

	assert_eq!(res.status, 404);
	assert_eq!(res.json()["status"], 404);
}

#[tokio::test]
async fn text_and_html_content_types() {
	let server = functional_server().await;

	let text = server.get("/api/text").await;
	assert_eq!(text.status, 200);
	assert!(text
		.header("content-type")
		.unwrap_or_default()
		.contains("text/plain"));
	assert_eq!(text.text(), "plain text body");

	let html = server.get("/api/html").await;
	assert_eq!(html.status, 200);
	assert!(html
		.header("content-type")
		.unwrap_or_default()
		.contains("text/html"));
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
		.configure(MurServerConfig::default().no_logging().body_size_limit(4096))
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
		.add_middleware(
			MurThrottler::new()
				.global()
				.requests(3)
				.per_minutes(1),
		)
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
	assert!(res
		.header("access-control-allow-methods")
		.unwrap_or_default()
		.contains("GET"));
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
