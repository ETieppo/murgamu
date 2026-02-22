use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ContentSecurityPolicy {
	directives: HashMap<String, String>,
	report_only: bool,
}

impl ContentSecurityPolicy {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn report_only(mut self) -> Self {
		self.report_only = true;
		self
	}

	pub fn default_src(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("default-src".to_string(), value.into());
		self
	}

	pub fn script_src(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("script-src".to_string(), value.into());
		self
	}

	pub fn style_src(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("style-src".to_string(), value.into());
		self
	}

	pub fn img_src(mut self, value: impl Into<String>) -> Self {
		self.directives.insert("img-src".to_string(), value.into());
		self
	}

	pub fn font_src(mut self, value: impl Into<String>) -> Self {
		self.directives.insert("font-src".to_string(), value.into());
		self
	}

	pub fn connect_src(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("connect-src".to_string(), value.into());
		self
	}

	pub fn media_src(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("media-src".to_string(), value.into());
		self
	}

	pub fn object_src(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("object-src".to_string(), value.into());
		self
	}

	pub fn frame_src(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("frame-src".to_string(), value.into());
		self
	}

	pub fn frame_ancestors(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("frame-ancestors".to_string(), value.into());
		self
	}

	pub fn base_uri(mut self, value: impl Into<String>) -> Self {
		self.directives.insert("base-uri".to_string(), value.into());
		self
	}

	pub fn form_action(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("form-action".to_string(), value.into());
		self
	}

	pub fn worker_src(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("worker-src".to_string(), value.into());
		self
	}

	pub fn manifest_src(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("manifest-src".to_string(), value.into());
		self
	}

	pub fn report_uri(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("report-uri".to_string(), value.into());
		self
	}

	pub fn report_to(mut self, value: impl Into<String>) -> Self {
		self.directives
			.insert("report-to".to_string(), value.into());
		self
	}

	pub fn upgrade_insecure_requests(mut self) -> Self {
		self.directives
			.insert("upgrade-insecure-requests".to_string(), String::new());
		self
	}

	pub fn block_all_mixed_content(mut self) -> Self {
		self.directives
			.insert("block-all-mixed-content".to_string(), String::new());
		self
	}

	pub fn directive(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
		self.directives.insert(name.into(), value.into());
		self
	}

	pub fn header_name(&self) -> &'static str {
		if self.report_only {
			"Content-Security-Policy-Report-Only"
		} else {
			"Content-Security-Policy"
		}
	}

	pub fn build(&self) -> String {
		self.directives
			.iter()
			.map(|(k, v)| {
				if v.is_empty() {
					k.clone()
				} else {
					format!("{} {}", k, v)
				}
			})
			.collect::<Vec<_>>()
			.join("; ")
	}
}

impl std::fmt::Display for ContentSecurityPolicy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.build())
	}
}
