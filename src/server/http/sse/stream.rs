use super::MurSseEvent;
use super::MurSseReceiver;

pub struct MurSseStream {
	pub inner: MurSseStreamInner,
}

pub enum MurSseStreamInner {
	Channel(MurSseReceiver),
	Events(Vec<MurSseEvent>),
}

impl MurSseStream {
	pub fn from_channel(receiver: MurSseReceiver) -> Self {
		Self {
			inner: MurSseStreamInner::Channel(receiver),
		}
	}

	pub fn from_events(events: Vec<MurSseEvent>) -> Self {
		Self {
			inner: MurSseStreamInner::Events(events),
		}
	}

	pub fn empty() -> Self {
		Self::from_events(Vec::new())
	}

	pub fn once(event: MurSseEvent) -> Self {
		Self::from_events(vec![event])
	}
}
