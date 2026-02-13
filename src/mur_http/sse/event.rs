use hyper::body::Bytes;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct MurSseEvent {
	pub event_type: Option<String>,
	pub data: Option<String>,
	pub id: Option<String>,
	pub retry: Option<u64>,
	pub comment: Option<String>,
	pub custom_fields: HashMap<String, String>,
}

impl MurSseEvent {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_data<S: Into<String>>(data: S) -> Self {
		Self::new().data(data)
	}

	pub fn comment<S: Into<String>>(comment: S) -> Self {
		Self {
			comment: Some(comment.into()),
			..Default::default()
		}
	}

	pub fn keep_alive() -> Self {
		Self::comment("ping")
	}

	pub fn retry_interval(milliseconds: u64) -> Self {
		Self {
			retry: Some(milliseconds),
			..Default::default()
		}
	}

	pub fn event<S: Into<String>>(mut self, event_type: S) -> Self {
		self.event_type = Some(event_type.into());
		self
	}

	pub fn data<S: Into<String>>(mut self, data: S) -> Self {
		self.data = Some(data.into());
		self
	}

	pub fn json<T: serde::Serialize>(self, value: &T) -> Result<Self, serde_json::Error> {
		let json_str = serde_json::to_string(value)?;
		Ok(self.data(json_str))
	}

	pub fn id<S: Into<String>>(mut self, id: S) -> Self {
		self.id = Some(id.into());
		self
	}

	pub fn retry_ms(mut self, milliseconds: u64) -> Self {
		self.retry = Some(milliseconds);
		self
	}

	pub fn retry_duration(self, duration: Duration) -> Self {
		self.retry_ms(duration.as_millis() as u64)
	}

	pub fn with_comment<S: Into<String>>(mut self, comment: S) -> Self {
		self.comment = Some(comment.into());
		self
	}

	pub fn field<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
		self.custom_fields.insert(key.into(), value.into());
		self
	}

	pub fn is_empty(&self) -> bool {
		self.event_type.is_none()
			&& self.data.is_none()
			&& self.id.is_none()
			&& self.retry.is_none()
			&& self.comment.is_none()
			&& self.custom_fields.is_empty()
	}

	pub fn to_bytes(&self) -> Bytes {
		Bytes::from(self.to_string())
	}
}

impl std::fmt::Display for MurSseEvent {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(ref comment) = self.comment {
			for line in comment.lines() {
				writeln!(f, ": {}", line)?;
			}
		}

		if let Some(ref event_type) = self.event_type {
			writeln!(f, "event: {}", event_type)?;
		}

		if let Some(ref id) = self.id {
			writeln!(f, "id: {}", id)?;
		}

		if let Some(retry) = self.retry {
			writeln!(f, "retry: {}", retry)?;
		}

		for (key, value) in &self.custom_fields {
			writeln!(f, "{}: {}", key, value)?;
		}

		if let Some(ref data) = self.data {
			for line in data.lines() {
				writeln!(f, "data: {}", line)?;
			}

			if data.is_empty() {
				writeln!(f, "data:")?;
			}
		}

		writeln!(f)?;
		Ok(())
	}
}

impl From<&str> for MurSseEvent {
	fn from(data: &str) -> Self {
		Self::with_data(data)
	}
}

impl From<String> for MurSseEvent {
	fn from(data: String) -> Self {
		Self::with_data(data)
	}
}
