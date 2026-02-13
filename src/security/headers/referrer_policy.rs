#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferrerPolicy {
	NoReferrer,
	NoReferrerWhenDowngrade,
	Origin,
	OriginWhenCrossOrigin,
	SameOrigin,
	StrictOrigin,
	StrictOriginWhenCrossOrigin,
	UnsafeUrl,
}

impl ReferrerPolicy {
	pub fn as_str(&self) -> &'static str {
		match self {
			ReferrerPolicy::NoReferrer => "no-referrer",
			ReferrerPolicy::NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
			ReferrerPolicy::Origin => "origin",
			ReferrerPolicy::OriginWhenCrossOrigin => "origin-when-cross-origin",
			ReferrerPolicy::SameOrigin => "same-origin",
			ReferrerPolicy::StrictOrigin => "strict-origin",
			ReferrerPolicy::StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
			ReferrerPolicy::UnsafeUrl => "unsafe-url",
		}
	}
}

impl std::fmt::Display for ReferrerPolicy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
