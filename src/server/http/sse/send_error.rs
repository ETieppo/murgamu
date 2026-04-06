#[derive(Debug, Clone)]
pub enum MurSseSendError {
	Closed,
	Full,
	SerializeError,
}

impl std::fmt::Display for MurSseSendError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Closed => write!(f, "SSE channel closed"),
			Self::Full => write!(f, "SSE channel buffer full"),
			Self::SerializeError => write!(f, "Failed to serialize SSE data"),
		}
	}
}
impl std::error::Error for MurSseSendError {}
