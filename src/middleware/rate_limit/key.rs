use crate::mur_http::request::MurRequestContext;
use std::sync::Arc;

type RateLimitCustomType = Arc<dyn Fn(&MurRequestContext) -> Option<String> + Send + Sync>;

#[derive(Clone)]
pub enum RateLimitKey {
	Ip,
	Header(String),
	BearerToken,
	IpAndHeader(String),
	Custom(RateLimitCustomType),
	Global,
}

impl std::fmt::Debug for RateLimitKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			RateLimitKey::Ip => write!(f, "Ip"),
			RateLimitKey::Header(name) => write!(f, "Header({:?})", name),
			RateLimitKey::BearerToken => write!(f, "BearerToken"),
			RateLimitKey::IpAndHeader(name) => write!(f, "IpAndHeader({:?})", name),
			RateLimitKey::Custom(_) => write!(f, "Custom(<function>)"),
			RateLimitKey::Global => write!(f, "Global"),
		}
	}
}

impl RateLimitKey {
	pub fn extract(&self, ctx: &MurRequestContext) -> Option<String> {
		match self {
			RateLimitKey::Ip => Self::extract_ip(ctx),
			RateLimitKey::Header(name) => ctx.header(name).map(|s| s.to_string()),
			RateLimitKey::BearerToken => Self::extract_bearer_token(ctx),
			RateLimitKey::IpAndHeader(name) => {
				let ip = Self::extract_ip(ctx)?;
				let header = ctx.header(name).map(|s| s.to_string()).unwrap_or_default();
				Some(format!("{}:{}", ip, header))
			}
			RateLimitKey::Custom(f) => f(ctx),
			RateLimitKey::Global => Some("__global__".to_string()),
		}
	}

	fn extract_ip(ctx: &MurRequestContext) -> Option<String> {
		if let Some(forwarded) = ctx.header("x-forwarded-for") {
			return forwarded.split(',').next().map(|s| s.trim().to_string());
		}

		if let Some(real_ip) = ctx.header("x-real-ip") {
			return Some(real_ip.to_string());
		}

		if let Some(cf_ip) = ctx.header("cf-connecting-ip") {
			return Some(cf_ip.to_string());
		}

		Some("unknown".to_string())
	}

	fn extract_bearer_token(ctx: &MurRequestContext) -> Option<String> {
		ctx.header("authorization").and_then(|auth| {
			if auth.to_lowercase().starts_with("bearer ") {
				Some(auth[7..].to_string())
			} else {
				None
			}
		})
	}
}
