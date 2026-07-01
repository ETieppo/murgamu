//! Public-API unit tests that don't need a running server.
//!
//! These lock down the behaviour of the framework's building blocks —
//! encoding/decoding, cookies, the error type, the open-redirect guard, and
//! the rate-limiter storage algorithms — including their security-relevant
//! edge cases.

use std::time::Duration;

use http_body_util::BodyExt;
use murgamu::core::utils::MurCodec;
use murgamu::server::middleware::rate_limit::{
	InMemoryStore, MurThrottlerStore, SlidingWindowStore, TokenBucketStore,
};
use murgamu::{MurCookie, MurError, MurResponder, SameSite};

// ===========================================================================
// MurCodec
// ===========================================================================

#[test]
fn base64_round_trips_text_and_binary() {
	for sample in [
		&b""[..],
		&b"f"[..],
		&b"fo"[..],
		&b"foo"[..],
		&b"hello world"[..],
		&[0u8, 1, 2, 250, 251, 255][..],
	] {
		let encoded = MurCodec::base64_encode(sample);
		let decoded = MurCodec::base64_decode(&encoded).expect("decodes");
		assert_eq!(decoded, sample, "round trip failed for {sample:?}");
	}
}

#[test]
fn base64_decode_rejects_invalid_input() {
	// Characters outside the alphabet must error, not silently decode.
	assert!(MurCodec::base64_decode("!!!!").is_err());
	assert!(MurCodec::base64_decode("aGVsbG8@").is_err());
	// Non-ASCII bytes.
	assert!(MurCodec::base64_decode("café").is_err());
}

#[test]
fn url_encode_decode_round_trip() {
	let raw = "a b/c?d=e&f#g+h%";
	let encoded = MurCodec::url_encode(raw);
	assert!(!encoded.contains(' '));
	let decoded = MurCodec::url_decode(&encoded).expect("decodes");
	assert_eq!(decoded, raw);
}

#[test]
fn url_decode_rejects_invalid_utf8() {
	// %FF decodes to the byte 0xFF, which is not valid UTF-8 on its own.
	assert!(MurCodec::url_decode("%FF").is_err());
}

// ===========================================================================
// MurCookie
// ===========================================================================

#[test]
fn cookie_default_is_lax_without_flags() {
	let header = MurCookie::new("k", "v").to_header_value();
	assert!(header.starts_with("k=v"));
	assert!(header.contains("SameSite=Lax"));
	assert!(!header.contains("Secure"));
	assert!(!header.contains("HttpOnly"));
}

#[test]
fn cookie_renders_all_attributes() {
	let header = MurCookie::new("sid", "abc")
		.path("/")
		.domain("example.com")
		.max_age(3600)
		.secure()
		.http_only()
		.same_site(SameSite::None)
		.to_header_value();

	assert!(header.contains("sid=abc"));
	assert!(header.contains("Path=/"));
	assert!(header.contains("Domain=example.com"));
	assert!(header.contains("Max-Age=3600"));
	assert!(header.contains("Secure"));
	assert!(header.contains("HttpOnly"));
	assert!(header.contains("SameSite=None"));
}

// ===========================================================================
// MurError
// ===========================================================================

#[test]
fn error_status_and_kind_mapping() {
	use http::StatusCode;
	assert_eq!(
		MurError::not_found("x").status_code(),
		StatusCode::NOT_FOUND
	);
	assert_eq!(
		MurError::unauthorized("x").status_code(),
		StatusCode::UNAUTHORIZED
	);
	assert_eq!(
		MurError::forbidden("x").status_code(),
		StatusCode::FORBIDDEN
	);
	assert_eq!(
		MurError::bad_request("x").status_code(),
		StatusCode::BAD_REQUEST
	);
	assert_eq!(MurError::conflict("x").status_code(), StatusCode::CONFLICT);
	assert_eq!(
		MurError::too_many_requests("x").status_code(),
		StatusCode::TOO_MANY_REQUESTS
	);
	assert_eq!(
		MurError::payload_too_large("x").status_code(),
		StatusCode::PAYLOAD_TOO_LARGE
	);
	assert_eq!(MurError::not_found("x").kind(), "not_found");
}

#[tokio::test]
async fn internal_error_response_does_not_leak_message() {
	let resp = MurError::internal("connection string with password").into_response();
	assert_eq!(resp.status(), 500);
	let body = resp.into_body().collect().await.unwrap().to_bytes();
	let text = String::from_utf8_lossy(&body);
	assert!(!text.contains("password"), "leaked internal detail: {text}");
	assert!(text.contains("Internal Server Error"));
}

#[tokio::test]
async fn client_error_response_includes_message() {
	let resp = MurError::not_found("user 42 missing").into_response();
	assert_eq!(resp.status(), 404);
	let body = resp.into_body().collect().await.unwrap().to_bytes();
	let text = String::from_utf8_lossy(&body);
	// Client errors are safe to echo back.
	assert!(text.contains("user 42 missing"), "body: {text}");
}

// ===========================================================================
// Open-redirect protection (MurResponder)
// ===========================================================================

#[test]
fn responder_allows_safe_redirect() {
	let status = MurResponder::redirect("/dashboard").unwrap().status();
	assert_eq!(status, 302);
}

#[test]
fn responder_blocks_dangerous_redirect_schemes() {
	for evil in [
		"javascript:alert(1)",
		"JavaScript:alert(1)",
		"data:text/html,<script>",
		"vbscript:msgbox(1)",
	] {
		let status = MurResponder::redirect(evil).unwrap().status();
		assert_eq!(status, 400, "scheme should be rejected: {evil}");
	}
}

// ===========================================================================
// Rate-limiter storage algorithms
// ===========================================================================

const WINDOW: Duration = Duration::from_secs(60);

#[test]
fn fixed_window_allows_quota_then_blocks() {
	let store = InMemoryStore::new();
	for _ in 0..3 {
		let (allowed, _, _) = store.check_and_update("k", 3, WINDOW);
		assert!(allowed);
	}
	let (allowed, remaining, _) = store.check_and_update("k", 3, WINDOW);
	assert!(!allowed, "4th request must be blocked");
	assert_eq!(remaining, 0);
}

#[test]
fn fixed_window_keys_are_independent() {
	let store = InMemoryStore::new();
	// Exhaust key "a".
	for _ in 0..3 {
		store.check_and_update("a", 3, WINDOW);
	}
	assert!(!store.check_and_update("a", 3, WINDOW).0);
	// Key "b" still has full quota.
	assert!(store.check_and_update("b", 3, WINDOW).0);
}

#[test]
fn fixed_window_reset_restores_quota() {
	let store = InMemoryStore::new();
	for _ in 0..3 {
		store.check_and_update("k", 3, WINDOW);
	}
	assert!(!store.check_and_update("k", 3, WINDOW).0);
	store.reset("k");
	assert!(
		store.check_and_update("k", 3, WINDOW).0,
		"reset should restore quota"
	);
}

#[test]
fn token_bucket_allows_quota_then_blocks() {
	let store = TokenBucketStore::new();
	for _ in 0..3 {
		assert!(store.check_and_update("k", 3, WINDOW).0);
	}
	assert!(
		!store.check_and_update("k", 3, WINDOW).0,
		"bucket should be empty"
	);
}

#[test]
fn token_bucket_refills_over_time() {
	// Small window so a brief sleep refills at least one token.
	let store = TokenBucketStore::new();
	let window = Duration::from_millis(300);
	for _ in 0..2 {
		assert!(store.check_and_update("k", 2, window).0);
	}
	assert!(!store.check_and_update("k", 2, window).0);

	std::thread::sleep(Duration::from_millis(400));
	assert!(
		store.check_and_update("k", 2, window).0,
		"a token should have refilled after the window elapsed"
	);
}

#[test]
fn sliding_window_allows_quota_then_blocks() {
	let store = SlidingWindowStore::new();
	for _ in 0..3 {
		assert!(store.check_and_update("k", 3, WINDOW).0);
	}
	assert!(!store.check_and_update("k", 3, WINDOW).0);
}
