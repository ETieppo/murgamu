use super::MurSseConfig;
use super::MurSseEvent;
use crate::server::aliases::MurRes;
use crate::server::error::MurError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::{CACHE_CONTROL, CONNECTION, CONTENT_TYPE};
use hyper::{Response, StatusCode};
use std::fmt::Write;
use std::time::Duration;

#[derive(Clone)]
pub struct MurSse {
	pub config: MurSseConfig,
}

impl MurSse {
	pub fn new() -> Self {
		Self {
			config: MurSseConfig::default(),
		}
	}

	pub fn with_config(config: MurSseConfig) -> Self {
		Self { config }
	}

	pub fn keep_alive(mut self, enabled: bool) -> Self {
		self.config.keep_alive = enabled;
		self
	}

	pub fn keep_alive_interval(mut self, interval: Duration) -> Self {
		self.config.keep_alive_interval = interval;
		self
	}

	pub fn retry_interval(mut self, interval: Duration) -> Self {
		self.config.retry_interval = Some(interval);
		self
	}

	pub fn no_retry_interval(mut self) -> Self {
		self.config.retry_interval = None;
		self
	}

	pub fn header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
		self.config.headers.insert(key.into(), value.into());
		self
	}

	pub fn events(self, events: Vec<MurSseEvent>) -> MurRes {
		self.build_response(events)
	}

	pub fn event(self, event: MurSseEvent) -> MurRes {
		self.events(vec![event])
	}

	pub fn data<S: Into<String>>(self, data: S) -> MurRes {
		self.event(MurSseEvent::with_data(data))
	}

	pub fn json<T: serde::Serialize>(self, value: &T) -> MurRes {
		match MurSseEvent::new().json(value) {
			Ok(event) => self.event(event),
			Err(e) => Err(MurError::Internal(e.to_string())),
		}
	}

	fn build_response(self, events: Vec<MurSseEvent>) -> MurRes {
		let mut body = String::new();

		if let Some(retry) = self.config.retry_interval {
			let _ = writeln!(body, "retry: {}\n", retry.as_millis());
		}

		for event in events {
			body.push_str(&event.to_string());
		}

		let mut response = Response::builder()
			.status(StatusCode::OK)
			.header(CONTENT_TYPE, "text/event-stream")
			.header(CACHE_CONTROL, "no-cache")
			.header(CONNECTION, "keep-alive")
			.header("X-Accel-Buffering", "no");

		for (key, value) in &self.config.headers {
			response = response.header(key.as_str(), value.as_str());
		}

		response
			.body(Full::new(Bytes::from(body)))
			.map_err(|e| MurError::Internal(e.to_string()))
	}

	pub fn config(&self) -> &MurSseConfig {
		&self.config
	}
}

impl Default for MurSse {
	fn default() -> Self {
		Self::new()
	}
}
