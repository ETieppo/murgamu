use super::MurSse;
use super::MurSseChannel;
use super::MurSseEvent;
use super::MurSseReceiver;
use super::MurSseSender;
use crate::types::MurRes;

pub fn mur_sse_data<S: Into<String>>(data: S) -> MurRes {
	MurSse::new().data(data)
}

pub fn mur_sse_event<S1: Into<String>, S2: Into<String>>(event_type: S1, data: S2) -> MurRes {
	MurSse::new().event(MurSseEvent::new().event(event_type).data(data))
}

pub fn mur_sse_json<T: serde::Serialize>(value: &T) -> MurRes {
	MurSse::new().json(value)
}

pub fn mur_sse_channel(buffer_size: usize) -> (MurSseSender, MurSseReceiver) {
	MurSseChannel::new_channel(buffer_size)
}

pub fn mur_sse_headers() -> Vec<(&'static str, &'static str)> {
	vec![
		("Content-Type", "text/event-stream"),
		("Cache-Control", "no-cache"),
		("Connection", "keep-alive"),
		("X-Accel-Buffering", "no"),
	]
}
