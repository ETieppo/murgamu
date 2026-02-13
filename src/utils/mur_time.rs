pub struct MurTime;

impl MurTime {
	pub fn now() -> String {
		chrono::Utc::now().to_rfc3339()
	}

	pub fn timestamp_ms() -> u64 {
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_millis() as u64)
			.unwrap_or(0)
	}

	pub fn timestamp_secs() -> u64 {
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_secs())
			.unwrap_or(0)
	}

	pub fn timestamp_iso() -> String {
		chrono::Utc::now().to_rfc3339()
	}
}
