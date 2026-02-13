use super::MurSseEvent;
use tokio::sync::mpsc;

pub struct MurSseReceiver {
	pub rx: mpsc::Receiver<MurSseEvent>,
}

impl MurSseReceiver {
	pub async fn recv(&mut self) -> Option<MurSseEvent> {
		self.rx.recv().await
	}
}
