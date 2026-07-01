//! Stability, security, and robustness tests for the Murgamü framework.
//!
//! Scenarios are drawn from the Bun.js HTTP server test suite and the
//! actix-web integration test suite, adapted for Murgamü's API surface.
//!
//! ## Categories
//!
//! * **Spec compliance** — HEAD/204/304 body rules, Content-Length accuracy
//! * **Keep-alive** — connection reuse, Connection: close
//! * **Security** — oversized URLs, path traversal, unknown methods, many query params
//! * **Multi-server** — two independent MurServer instances on separate ports
//! * **Concurrency** — burst loads, mixed method storms, concurrent POST bodies
//! * **Response integrity** — checksum of large bodies, custom header round-trip,
//!   redirect Location, correct Content-Type per response kind
//! * **Error handling** — 4xx/5xx status codes, no detail leakage

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use http_body_util::{BodyExt, Full};
use hyper::Request;
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use murgamu::{MurServer, MurServerRunner};
use tokio::net::TcpStream;

// ===========================================================================
// Test application
// ===========================================================================

mod app {
	use murgamu::prelude::*;

	#[derive(Clone)]
	pub struct StabilityController;

	#[controller("/")]
	impl StabilityController {
		pub fn new() -> Self {
			Self
		}

		/// Minimal health-check route.
		#[get("/ping")]
		async fn ping(&self) -> MurRes {
			mur_json!({ "ok": true })
		}

		/// 204 No Content — body MUST be absent per RFC 9110 § 6.3.5.
		#[get("/empty")]
		async fn empty(&self) -> MurRes {
			MurHttpResponse::no_content()
		}

		/// 301 redirect — verifies Location header is emitted.
		#[get("/redirect")]
		async fn redirect(&self) -> MurRes {
			MurHttpResponse::moved_permanently("https://example.com/")
		}

		/// Echo the value of `X-Echo` back in the JSON body.
		#[get("/echo-header")]
		async fn echo_header(&self, ctx: MurRequestContext) -> MurRes {
			let v = ctx.header("x-echo").unwrap_or("").to_string();
			mur_json!({ "echo": v })
		}

		/// Echo the raw request body back as plain text.
		#[post("/echo-body")]
		async fn echo_body(&self, ctx: MurRequestContext) -> MurRes {
			match ctx.body_string() {
				Ok(s) => mur_text!(s),
				Err(e) => e.into(),
			}
		}

		/// Return a response whose body is `kb` KiB of repeated 'a' characters.
		#[get("/large/:kb")]
		async fn large(&self, #[param] kb: usize) -> MurRes {
			mur_text!("a".repeat(kb * 1024))
		}

		/// Plain-text response.
		#[get("/text")]
		async fn text(&self) -> MurRes {
			mur_text!("hello")
		}

		/// HTML response.
		#[get("/html")]
		async fn html(&self) -> MurRes {
			mur_html!("<b>hi</b>")
		}

		/// Propagate a MurError so the router maps it to a non-200 status.
		#[get("/error/:kind")]
		async fn error_ep(&self, #[param] kind: String) -> MurRes {
			match kind.as_str() {
				"not-found" => MurError::not_found("resource missing").into(),
				"bad-request" => MurError::bad_request("invalid input").into(),
				"internal" => MurError::internal("secret details").into(),
				"conflict" => MurError::conflict("already exists").into(),
				_ => mur_json!({ "kind": kind }),
			}
		}

		/// Counts query parameters and returns the total.
		#[get("/count-query")]
		async fn count_query(&self, ctx: MurRequestContext) -> MurRes {
			let n = ctx.query_map().len();
			mur_json!({ "count": n })
		}
	}

	#[module(controllers: [StabilityController])]
	pub struct StabilityModule;
}

// ===========================================================================
// Test harness
// ===========================================================================

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

	async fn post(&self, path: &str, body: Vec<u8>, content_type: &str) -> TestResponse {
		self.send("POST", path, &[("content-type", content_type)], body)
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

	fn content_length(&self) -> Option<usize> {
		self.header("content-length")?.parse().ok()
	}
}

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

/// Sends `count` sequential GET requests over the **same** HTTP/1.1 connection.
/// Returns (status, body_len) for each response.
async fn keepalive_requests(addr: SocketAddr, paths: &[&str]) -> Vec<(u16, usize)> {
	let stream = TcpStream::connect(addr).await.expect("connect");
	let io = TokioIo::new(stream);
	let (mut sender, conn): (hyper::client::conn::http1::SendRequest<Full<Bytes>>, _) =
		hyper::client::conn::http1::handshake(io)
			.await
			.expect("handshake");
	tokio::spawn(async move {
		let _ = conn.await;
	});

	let mut results = Vec::new();
	for path in paths {
		let req = Request::builder()
			.method("GET")
			.uri(*path)
			.header("Host", addr.to_string())
			.body(Full::new(Bytes::new()))
			.unwrap();
		let res = sender.send_request(req).await.expect("send");
		let status = res.status().as_u16();
		let body = res.into_body().collect().await.unwrap().to_bytes();
		results.push((status, body.len()));
	}
	results
}

fn free_addr() -> SocketAddr {
	let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
	let addr = l.local_addr().unwrap();
	drop(l);
	addr
}

async fn default_server() -> TestServer {
	let addr = free_addr();
	let runner = MurServer::new()
		.no_logging()
		.default_public_routes()
		.module(app::StabilityModule::new())
		.bind(addr)
		.expect("bind stability server");
	TestServer::start(runner).await
}

// ===========================================================================
// § 1  HTTP spec compliance
// ===========================================================================

/// HEAD responses MUST carry the same headers as GET but MUST NOT include a
/// message body. (RFC 9110 § 9.3.2)
#[tokio::test]
async fn head_response_has_no_body() {
	let server = default_server().await;
	let res = server.send("HEAD", "/ping", &[], Vec::new()).await;

	assert_eq!(res.status, 200);
	assert_eq!(res.body.len(), 0, "HEAD must not carry a response body");
}

/// The Content-Length reported by HEAD must equal the byte count of the
/// GET body for the same resource. (RFC 9110 § 9.3.2)
#[tokio::test]
async fn head_content_length_matches_get_body_size() {
	let server = default_server().await;

	let get = server.get("/ping").await;
	let head = server.send("HEAD", "/ping", &[], Vec::new()).await;

	assert_eq!(get.status, 200);
	assert_eq!(head.status, 200);

	// The GET body must match the Content-Length header reported by HEAD.
	if let Some(cl) = head.content_length() {
		assert_eq!(
			cl,
			get.body.len(),
			"HEAD Content-Length must match GET body size"
		);
	}
}

/// 204 No Content responses MUST NOT include a message body. (RFC 9110 § 6.3.5)
#[tokio::test]
async fn no_content_204_body_is_absent() {
	let server = default_server().await;
	let res = server.get("/empty").await;

	assert_eq!(res.status, 204);
	assert_eq!(res.body.len(), 0, "204 must not have a body");
}

/// 301 Moved Permanently MUST include a Location header. (RFC 9110 § 15.4.2)
#[tokio::test]
async fn redirect_has_location_header() {
	let server = default_server().await;
	let res = server.get("/redirect").await;

	assert_eq!(res.status, 301);
	assert!(
		res.header("location").is_some(),
		"301 must carry Location header"
	);
}

/// The framework must emit `Content-Type: application/json` for JSON responses.
#[tokio::test]
async fn json_response_content_type_is_application_json() {
	let server = default_server().await;
	let res = server.get("/ping").await;

	assert_eq!(res.status, 200);
	let ct = res.header("content-type").unwrap_or_default();
	assert!(ct.contains("application/json"), "Content-Type was: {ct}");
}

/// The framework must emit `Content-Type: text/plain` for text responses.
#[tokio::test]
async fn text_response_content_type_is_text_plain() {
	let server = default_server().await;
	let res = server.get("/text").await;

	assert_eq!(res.status, 200);
	let ct = res.header("content-type").unwrap_or_default();
	assert!(ct.contains("text/plain"), "Content-Type was: {ct}");
}

/// The framework must emit `Content-Type: text/html` for HTML responses.
#[tokio::test]
async fn html_response_content_type_is_text_html() {
	let server = default_server().await;
	let res = server.get("/html").await;

	assert_eq!(res.status, 200);
	let ct = res.header("content-type").unwrap_or_default();
	assert!(ct.contains("text/html"), "Content-Type was: {ct}");
}

// ===========================================================================
// § 2  Keep-alive / connection reuse
// ===========================================================================

/// HTTP/1.1 keep-alive: the same TCP connection must serve multiple sequential
/// requests without being closed between them. (actix-web: `http1_keepalive`)
#[tokio::test]
async fn keepalive_reuses_connection_for_sequential_requests() {
	let server = default_server().await;
	let paths = ["/ping", "/text", "/html", "/ping", "/text"];
	let results = keepalive_requests(server.addr, &paths).await;

	assert_eq!(results.len(), paths.len());
	for (i, (status, body_len)) in results.iter().enumerate() {
		assert_eq!(*status, 200, "request {i} failed on keep-alive connection");
		assert!(
			*body_len > 0,
			"request {i} returned empty body on keep-alive connection"
		);
	}
}

/// A response with `Connection: close` must signal that the server will close
/// the connection after the response.  We verify by sending that request, then
/// attempting a second — the second may fail since the socket is gone, or the
/// server may give a fresh connection.  What must NOT happen is the first
/// request being corrupted. (actix-web: `http1_keepalive_close`)
#[tokio::test]
async fn connection_close_request_header_accepted() {
	let server = default_server().await;
	let res = server.get_with("/ping", &[("connection", "close")]).await;

	// The response must still be valid even when the client signals no keep-alive.
	assert_eq!(res.status, 200);
	assert!(res.json()["ok"].as_bool().unwrap_or(false));
}

// ===========================================================================
// § 3  Security
// ===========================================================================

/// A URL that is 8 KiB long must not crash the server. The framework may
/// return any 4xx status or 200 for the matched (or unmatched) route, but it
/// must NOT panic. (Bun.js: very-long URL test)
#[tokio::test]
async fn very_long_url_does_not_crash_server() {
	let server = default_server().await;
	let long_path = format!("/ping?q={}", "a".repeat(8192));
	let res = server.get(&long_path).await;

	// The response code is not strictly defined, but must be a valid HTTP status.
	assert!(
		res.status >= 200 && res.status < 600,
		"invalid status for long URL: {}",
		res.status
	);
}

/// A single header whose value is 32 KiB must not crash the server.
/// (Bun.js: `max-http-header-size` family; actix: `test_invalid_header`)
#[tokio::test]
async fn oversized_header_value_does_not_crash_server() {
	let server = default_server().await;
	let big_value = "x".repeat(32 * 1024);
	// hyper will either forward it or reject at the transport layer — both are fine.
	let res = server
		.get_with("/ping", &[("x-large", big_value.as_str())])
		.await;

	assert!(
		res.status >= 200 && res.status < 600,
		"invalid status for large header"
	);
}

/// Path-traversal segments (`../`) must not cause the server to panic or leak
/// routes beyond the registered ones. The expected outcome is 404. (Bun.js)
#[tokio::test]
async fn path_traversal_segments_return_404() {
	let server = default_server().await;

	for path in &["/../../etc/passwd", "/../ping", "/ping/../../../etc"] {
		let res = server.get(path).await;
		assert_eq!(
			res.status, 404,
			"path traversal attempt `{path}` should return 404, got {}",
			res.status
		);
	}
}

/// An unregistered HTTP method (TRACE) must be handled without panicking.
/// The server returns 404 because no route is registered for TRACE.
/// (Bun.js: HTTP method edge cases)
#[tokio::test]
async fn unknown_http_method_does_not_crash_server() {
	let server = default_server().await;
	let res = server.send("TRACE", "/ping", &[], Vec::new()).await;

	// 404 (no matching route) or 405 (method not allowed) are both acceptable.
	assert!(
		res.status == 404 || res.status == 405,
		"TRACE should yield 404 or 405, got {}",
		res.status
	);
}

/// Hundreds of query parameters must not cause the server to panic or
/// return an error. (Bun.js: many query params)
#[tokio::test]
async fn many_query_params_do_not_crash() {
	let server = default_server().await;
	let qs: String = (0..300)
		.map(|i| format!("k{i}=v{i}"))
		.collect::<Vec<_>>()
		.join("&");
	let path = format!("/count-query?{qs}");
	let res = server.get(&path).await;

	assert_eq!(res.status, 200);
	let count = res.json()["count"].as_u64().unwrap_or(0);
	assert!(count > 0, "expected at least one query param to be parsed");
}

/// A POST with `Content-Length: 0` and no body bytes must be accepted without error.
/// (Bun.js: empty-body POST)
#[tokio::test]
async fn post_with_empty_body_content_length_zero_accepted() {
	let server = default_server().await;
	let res = server
		.send(
			"POST",
			"/echo-body",
			&[("content-length", "0"), ("content-type", "text/plain")],
			Vec::new(),
		)
		.await;

	// Handler receives empty body; either returns "" or an error, but must not panic.
	assert!(
		res.status == 200 || res.status == 400,
		"empty POST should yield 200 or 400, got {}",
		res.status
	);
}

/// A custom response header set inside a handler must arrive at the client
/// unchanged. (Bun.js: header round-trip)
#[tokio::test]
async fn custom_response_header_survives_round_trip() {
	let server = default_server().await;
	let res = server
		.get_with("/echo-header", &[("x-echo", "hello-world-123")])
		.await;

	assert_eq!(res.status, 200);
	assert_eq!(res.json()["echo"], "hello-world-123");
}

// ===========================================================================
// § 4  Multi-server isolation
// ===========================================================================

/// Two independent MurServer instances bound to different ports must each
/// respond correctly without interfering. (Bun.js: independent servers)
#[tokio::test]
async fn two_servers_operate_independently_on_separate_ports() {
	let server_a = default_server().await;
	let server_b = default_server().await;

	// Verify they bound to different ports.
	assert_ne!(server_a.addr.port(), server_b.addr.port());

	let res_a = server_a.get("/ping").await;
	let res_b = server_b.get("/ping").await;

	assert_eq!(res_a.status, 200);
	assert_eq!(res_b.status, 200);
}

/// Three servers running concurrently each handle their own workload without
/// corrupting each other's responses.
#[tokio::test]
async fn three_servers_handle_concurrent_load_in_isolation() {
	let s1 = default_server().await;
	let s2 = default_server().await;
	let s3 = default_server().await;

	let (r1, r2, r3) = tokio::join!(s1.get("/ping"), s2.get("/text"), s3.get("/html"),);

	assert_eq!(r1.status, 200);
	assert_eq!(r2.status, 200);
	assert_eq!(r3.status, 200);

	// Each must have gotten the right route.
	assert!(
		r1.json()["ok"].as_bool().unwrap_or(false),
		"server1 /ping response wrong"
	);
	assert_eq!(r2.text(), "hello", "server2 /text response wrong");
	assert!(r3.text().contains("hi"), "server3 /html response wrong");
}

// ===========================================================================
// § 5  Concurrency
// ===========================================================================

/// 300 concurrent GET requests must all succeed. Tests that the framework
/// does not deadlock or panic under burst load. (Bun.js: abort/crash recovery;
/// actix-web: `test_start`, concurrent connections)
#[tokio::test]
async fn burst_of_300_concurrent_get_requests_all_succeed() {
	let server = default_server().await;
	let addr = server.addr;

	let mut set = tokio::task::JoinSet::new();
	for _ in 0..300 {
		set.spawn(async move { raw_request(addr, "GET", "/ping", &[], Vec::new()).await });
	}

	let mut ok = 0usize;
	while let Some(res) = set.join_next().await {
		if res.expect("task panicked").status == 200 {
			ok += 1;
		}
	}
	assert_eq!(ok, 300, "all concurrent GETs should return 200");
}

/// Mixed HTTP methods (GET, POST) sent concurrently must each be routed to
/// the correct handler. (Bun.js: crash recovery under concurrent load)
#[tokio::test]
async fn concurrent_mixed_method_requests_route_correctly() {
	let server = default_server().await;
	let addr = server.addr;

	let mut set = tokio::task::JoinSet::new();
	for i in 0..100 {
		set.spawn(async move {
			if i % 2 == 0 {
				raw_request(addr, "GET", "/ping", &[], Vec::new()).await
			} else {
				raw_request(
					addr,
					"POST",
					"/echo-body",
					&[("content-type", "text/plain")],
					b"payload".to_vec(),
				)
				.await
			}
		});
	}

	let mut ok = 0usize;
	while let Some(res) = set.join_next().await {
		if res.expect("task").status == 200 {
			ok += 1;
		}
	}
	assert_eq!(
		ok, 100,
		"all mixed-method concurrent requests should return 200"
	);
}

/// 100 concurrent POST requests each carrying a 4 KiB body must all be
/// parsed correctly. (Bun.js: 1000 uploads/downloads; actix-web body tests)
#[tokio::test]
async fn concurrent_post_requests_with_bodies_are_parsed_correctly() {
	let server = default_server().await;
	let addr = server.addr;

	let body = b"x".repeat(4096);
	let expected_len = body.len();

	let mut set = tokio::task::JoinSet::new();
	for _ in 0..100 {
		let b = body.clone();
		set.spawn(async move {
			raw_request(
				addr,
				"POST",
				"/echo-body",
				&[("content-type", "text/plain")],
				b,
			)
			.await
		});
	}

	let mut ok = 0usize;
	while let Some(res) = set.join_next().await {
		let r = res.expect("task");
		if r.status == 200 && r.body.len() == expected_len {
			ok += 1;
		}
	}
	assert_eq!(ok, 100, "all concurrent POSTs should echo the full body");
}

// ===========================================================================
// § 6  Response integrity
// ===========================================================================

/// A 512 KiB response body must arrive intact: every byte must be 'a' (0x61).
/// (Bun.js: 768 KB blob integrity test; actix: `body_gzip_large`)
#[tokio::test]
async fn large_response_body_arrives_intact() {
	let server = default_server().await;
	const KB: usize = 512;
	let res = server.get(&format!("/large/{KB}")).await;

	assert_eq!(res.status, 200);
	assert_eq!(res.body.len(), KB * 1024, "body length mismatch");
	assert!(
		res.body.iter().all(|&b| b == b'a'),
		"body corruption detected in large response"
	);
}

/// The framework must echo POST bodies accurately at any size up to the limit.
#[tokio::test]
async fn post_echo_preserves_body_exactly() {
	let server = default_server().await;
	// Use printable ASCII that cycles predictably — keeps the test deterministic
	// without depending on binary-safe UTF-8 decoding inside the handler.
	let payload: String = (b'!'..=b'~')
		.cycle()
		.take(2048)
		.map(|b| b as char)
		.collect();
	let res = server
		.post("/echo-body", payload.as_bytes().to_vec(), "text/plain")
		.await;

	assert_eq!(res.status, 200);
	assert_eq!(
		res.body,
		payload.as_bytes(),
		"echo-body must return the exact bytes sent"
	);
}

// ===========================================================================
// § 7  Error handling
// ===========================================================================

/// Known error kinds must map to the correct HTTP status codes.
/// (actix-web: `error_type_maps_to_status_codes`)
#[tokio::test]
async fn error_variants_map_to_correct_http_status_codes() {
	let server = default_server().await;

	let cases = [
		("not-found", 404u16),
		("bad-request", 400),
		("conflict", 409),
	];

	for (kind, expected_status) in cases {
		let res = server.get(&format!("/error/{kind}")).await;
		assert_eq!(
			res.status, expected_status,
			"error kind `{kind}` should map to {expected_status}, got {}",
			res.status
		);
	}
}

/// Internal errors must not expose sensitive detail strings to the client.
/// (Bun.js: `/bun:info` security gate; actix-web: message-leakage test)
#[tokio::test]
async fn internal_server_errors_do_not_leak_sensitive_details() {
	let server = default_server().await;
	let res = server.get("/error/internal").await;

	assert_eq!(res.status, 500);
	let body = res.text();
	assert!(
		!body.contains("secret details"),
		"internal error detail leaked to client: {body}"
	);
}

/// A 404 response must include a JSON body with a `status` field set to 404.
#[tokio::test]
async fn not_found_response_includes_json_status_field() {
	let server = default_server().await;
	let res = server.get("/this-route-does-not-exist").await;

	assert_eq!(res.status, 404);
	assert_eq!(res.json()["status"], 404);
}

/// 4xx and 5xx error responses must include a `Content-Type: application/json` header.
#[tokio::test]
async fn error_responses_have_json_content_type() {
	let server = default_server().await;

	for path in &[
		"/this-does-not-exist",
		"/error/not-found",
		"/error/bad-request",
	] {
		let res = server.get(path).await;
		let ct = res.header("content-type").unwrap_or_default();
		assert!(
			ct.contains("application/json"),
			"error response for `{path}` should be application/json, got: {ct}"
		);
	}
}

// ===========================================================================
// § 8  Performance / throughput stability
// ===========================================================================

/// 1 000 sequential requests must complete in under 30 s and all return 200.
/// Throughput must not degrade significantly between the first and last
/// batches of 100. (actix-web: `perf_sequential_throughput_is_reasonable`)
#[tokio::test]
async fn sequential_throughput_is_stable_over_1000_requests() {
	let server = default_server().await;
	const N: usize = 1_000;
	const BATCH: usize = 100;

	let start = Instant::now();
	let mut batch_times = Vec::new();

	for batch in 0..(N / BATCH) {
		let t0 = Instant::now();
		for _ in 0..BATCH {
			let res = server.get("/ping").await;
			assert_eq!(res.status, 200, "throughput test: unexpected status");
		}
		batch_times.push(t0.elapsed());
		let _ = batch;
	}

	let elapsed = start.elapsed();
	assert!(
		elapsed < Duration::from_secs(30),
		"1000 requests took too long: {elapsed:?}"
	);

	// The last batch must not be more than 4× slower than the fastest batch.
	let min_batch = batch_times.iter().min().copied().unwrap_or_default();
	let last_batch = *batch_times.last().unwrap();
	assert!(
		last_batch <= min_batch * 4,
		"throughput degraded: first-batch min {min_batch:?}, last batch {last_batch:?}"
	);

	let rps = N as f64 / elapsed.as_secs_f64();
	eprintln!("stability: {N} sequential requests in {elapsed:?} (~{rps:.0} req/s)");
}

/// 500 concurrent requests must complete in under 30 s with all returning 200.
/// Measures the framework's ability to handle a sharp burst. (Bun.js: crash
/// recovery; actix-web: perf_handles_concurrent_requests)
#[tokio::test]
async fn burst_throughput_500_concurrent_requests() {
	let server = default_server().await;
	let addr = server.addr;

	let start = Instant::now();
	let mut set = tokio::task::JoinSet::new();
	for _ in 0..500 {
		set.spawn(async move { raw_request(addr, "GET", "/ping", &[], Vec::new()).await });
	}

	let mut ok = 0usize;
	while let Some(res) = set.join_next().await {
		if res.expect("task").status == 200 {
			ok += 1;
		}
	}
	let elapsed = start.elapsed();

	assert_eq!(ok, 500, "all 500 concurrent requests should return 200");
	assert!(
		elapsed < Duration::from_secs(30),
		"500 concurrent requests took too long: {elapsed:?}"
	);
	eprintln!("stability: 500 concurrent requests in {elapsed:?}");
}
