use super::MurSseEvent;
use super::MurSseSendError;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct MurSseSender {
	pub tx: mpsc::Sender<MurSseEvent>,
}

impl MurSseSender {
	pub async fn send(&self, event: MurSseEvent) -> Result<(), MurSseSendError> {
		self.tx
			.send(event)
			.await
			.map_err(|_| MurSseSendError::Closed)
	}

	pub fn try_send(&self, event: MurSseEvent) -> Result<(), MurSseSendError> {
		self.tx.try_send(event).map_err(|e| match e {
			mpsc::error::TrySendError::Full(_) => MurSseSendError::Full,
			mpsc::error::TrySendError::Closed(_) => MurSseSendError::Closed,
		})
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

	pub async fn send_json<T: serde::Serialize>(&self, value: &T) -> Result<(), MurSseSendError> {
		let event = MurSseEvent::new()
			.json(value)
			.map_err(|_| MurSseSendError::SerializeError)?;
		self.send(event).await
	}

	pub async fn ping(&self) -> Result<(), MurSseSendError> {
		self.send(MurSseEvent::keep_alive()).await
	}

	pub fn is_closed(&self) -> bool {
		self.tx.is_closed()
	}
}
