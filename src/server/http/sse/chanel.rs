use tokio::sync::mpsc;
pub struct MurSseChannel;
use super::MurSseReceiver;
use super::MurSseSender;

impl MurSseChannel {
	pub fn new_channel(buffer_size: usize) -> (MurSseSender, MurSseReceiver) {
		let (tx, rx) = mpsc::channel(buffer_size);
		(MurSseSender { tx }, MurSseReceiver { rx })
	}

	pub fn default_channel() -> (MurSseSender, MurSseReceiver) {
		Self::new_channel(256)
	}
}
