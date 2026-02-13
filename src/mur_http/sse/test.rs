use super::*;

#[test]
fn test_event_data_only() {
	let event = MurSseEvent::with_data("Hello, World!");
	let output = event.to_string();
	assert!(output.contains("data: Hello, World!"));
	assert!(output.ends_with("\n\n"));
}

#[test]
fn test_event_with_type() {
	let event = MurSseEvent::new().event("message").data("Hello");
	let output = event.to_string();
	assert!(output.contains("event: message"));
	assert!(output.contains("data: Hello"));
}

#[test]
fn test_event_with_id() {
	let event = MurSseEvent::new().id("12345").data("Hello");
	let output = event.to_string();
	assert!(output.contains("id: 12345"));
}

#[test]
fn test_event_with_retry() {
	let event = MurSseEvent::new().retry_ms(3000).data("Hello");
	let output = event.to_string();
	assert!(output.contains("retry: 3000"));
}

#[test]
fn test_event_comment() {
	let event = MurSseEvent::comment("this is a comment");
	let output = event.to_string();
	assert!(output.contains(": this is a comment"));
}

#[test]
fn test_event_multiline_data() {
	let event = MurSseEvent::with_data("Line 1\nLine 2\nLine 3");
	let output = event.to_string();
	assert!(output.contains("data: Line 1\n"));
	assert!(output.contains("data: Line 2\n"));
	assert!(output.contains("data: Line 3\n"));
}

#[test]
fn test_event_full() {
	let event = MurSseEvent::new()
		.event("notification")
		.id("msg-123")
		.retry_ms(5000)
		.data(r#"{"type": "alert"}"#);

	let output = event.to_string();
	assert!(output.contains("event: notification\n"));
	assert!(output.contains("id: msg-123\n"));
	assert!(output.contains("retry: 5000\n"));
	assert!(output.contains(r#"data: {"type": "alert"}"#));
}

#[test]
fn test_event_keep_alive() {
	let event = MurSseEvent::keep_alive();
	let output = event.to_string();
	assert!(output.contains(": ping"));
}

#[test]
fn test_event_from_string() {
	let event: MurSseEvent = "Hello".into();
	assert_eq!(event.data, Some("Hello".to_string()));

	let event: MurSseEvent = String::from("World").into();
	assert_eq!(event.data, Some("World".to_string()));
}

#[test]
fn test_event_is_empty() {
	let event = MurSseEvent::new();
	assert!(event.is_empty());

	let event = MurSseEvent::with_data("test");
	assert!(!event.is_empty());
}

#[test]
fn test_sse_config_default() {
	let config = MurSseConfig::default();
	assert!(config.keep_alive);
	assert_eq!(
		config.keep_alive_interval,
		std::time::Duration::from_secs(30)
	);
	assert!(config.retry_interval.is_some());
}

#[test]
fn test_sse_builder() {
	let sse = MurSse::new()
		.keep_alive(false)
		.retry_interval(std::time::Duration::from_secs(10))
		.header("X-Custom", "value");

	assert!(!sse.config.keep_alive);
	assert_eq!(
		sse.config.retry_interval,
		Some(std::time::Duration::from_secs(10))
	);
	assert!(sse.config.headers.contains_key("X-Custom"));
}

#[test]
fn test_sse_headers() {
	let headers = mur_sse_headers();
	assert!(headers
		.iter()
		.any(|(k, v)| *k == "Content-Type" && *v == "text/event-stream"));
	assert!(headers
		.iter()
		.any(|(k, v)| *k == "Cache-Control" && *v == "no-cache"));
}

#[tokio::test]
async fn test_sse_channel() {
	let (sender, mut receiver) = MurSseChannel::new_channel(10);
	sender.send_data("test").await.unwrap();

	let event = receiver.recv().await.unwrap();
	assert_eq!(event.data, Some("test".to_string()));
}

#[tokio::test]
async fn test_sse_sender_closed() {
	let (sender, receiver) = MurSseChannel::new_channel(10);
	drop(receiver);

	let result = sender.send_data("test").await;
	assert!(matches!(result, Err(MurSseSendError::Closed)));
}
