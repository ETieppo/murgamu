use super::MurSseEvent;
use super::MurSseSendError;
use super::MurSseSender;
use std::sync::Arc;

pub struct MurSseConnection {
	pub sender: MurSseSender,
	pub last_event_id: Arc<std::sync::Mutex<Option<String>>>,
	pub event_counter: Arc<std::sync::atomic::AtomicU64>,
}

impl MurSseConnection {
	pub fn new(sender: MurSseSender) -> Self {
		Self {
			sender,
			last_event_id: Arc::new(std::sync::Mutex::new(None)),
			event_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
		}
	}

	pub fn sender(&self) -> &MurSseSender {
		&self.sender
	}

	pub async fn send(&self, mut event: MurSseEvent) -> Result<(), MurSseSendError> {
		if event.id.is_none() {
			let id = self
				.event_counter
				.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
			event = event.id(id.to_string());
		}

		if let Some(ref id) = event.id
			&& let Ok(mut last_id) = self.last_event_id.lock()
		{
			*last_id = Some(id.clone());
		}

		self.sender.send(event).await
	}

	pub async fn send_data<S: Into<String>>(&self, data: S) -> Result<(), MurSseSendError> {
		self.send(MurSseEvent::with_data(data)).await
	}

	pub async fn send_event<S1: Into<String>, S2: Into<String>>(
		&self,
		event_type: S1,
		data: S2,
	) -> Result<(), MurSseSendError> {
		self.send(MurSseEvent::new().event(event_type).data(data))
			.await
	}

	pub fn last_event_id(&self) -> Option<String> {
		self.last_event_id
			.lock()
			.ok()
			.and_then(|guard| guard.clone())
	}

	pub fn is_closed(&self) -> bool {
		self.sender.is_closed()
	}
}

impl Clone for MurSseConnection {
	fn clone(&self) -> Self {
		Self {
			sender: self.sender.clone(),
			last_event_id: Arc::clone(&self.last_event_id),
			event_counter: Arc::clone(&self.event_counter),
		}
	}
}
