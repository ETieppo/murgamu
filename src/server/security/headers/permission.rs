use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct PermissionsPolicy {
	policies: HashMap<String, PermissionValue>,
}

#[derive(Debug, Clone)]
pub enum PermissionValue {
	None,
	All,
	Origins(Vec<String>),
}

impl PermissionsPolicy {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn deny(mut self, feature: impl Into<String>) -> Self {
		self.policies.insert(feature.into(), PermissionValue::None);
		self
	}

	pub fn allow_self(mut self, feature: impl Into<String>) -> Self {
		self.policies.insert(
			feature.into(),
			PermissionValue::Origins(vec!["self".to_string()]),
		);
		self
	}

	pub fn allow_all(mut self, feature: impl Into<String>) -> Self {
		self.policies.insert(feature.into(), PermissionValue::All);
		self
	}

	pub fn allow_origins(
		mut self,
		feature: impl Into<String>,
		origins: impl IntoIterator<Item = impl Into<String>>,
	) -> Self {
		self.policies.insert(
			feature.into(),
			PermissionValue::Origins(origins.into_iter().map(|o| o.into()).collect()),
		);
		self
	}

	pub fn disable_all(mut self) -> Self {
		let features = [
			"accelerometer",
			"ambient-light-sensor",
			"autoplay",
			"battery",
			"camera",
			"display-capture",
			"document-domain",
			"encrypted-media",
			"fullscreen",
			"geolocation",
			"gyroscope",
			"magnetometer",
			"microphone",
			"midi",
			"payment",
			"picture-in-picture",
			"publickey-credentials-get",
			"screen-wake-lock",
			"sync-xhr",
			"usb",
			"web-share",
			"xr-spatial-tracking",
		];

		for feature in features {
			self.policies
				.insert(feature.to_string(), PermissionValue::None);
		}
		self
	}

	pub fn build(&self) -> String {
		self.policies
			.iter()
			.map(|(feature, value)| match value {
				PermissionValue::None => format!("{}=()", feature),
				PermissionValue::All => format!("{}=*", feature),
				PermissionValue::Origins(origins) => {
					let origins_str = origins
						.iter()
						.map(|o| {
							if o == "self" {
								"self".to_string()
							} else {
								format!("\"{}\"", o)
							}
						})
						.collect::<Vec<_>>()
						.join(" ");
					format!("{}=({})", feature, origins_str)
				}
			})
			.collect::<Vec<_>>()
			.join(", ")
	}
}

impl std::fmt::Display for PermissionsPolicy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.build())
	}
}
