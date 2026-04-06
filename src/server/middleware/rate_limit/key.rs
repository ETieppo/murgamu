use crate::server::http::MurRequestContext;
use std::sync::Arc;

type MurThrottlerCustomType = Arc<dyn Fn(&MurRequestContext) -> Option<String> + Send + Sync>;

#[derive(Clone)]
pub enum MurThrottlerKey {
	Ip,
	Header(String),
	BearerToken,
	IpAndHeader(String),
	Custom(MurThrottlerCustomType),
	Global,
}

impl std::fmt::Debug for MurThrottlerKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MurThrottlerKey::Ip => write!(f, "Ip"),
			MurThrottlerKey::Header(name) => write!(f, "Header({:?})", name),
			MurThrottlerKey::BearerToken => write!(f, "BearerToken"),
			MurThrottlerKey::IpAndHeader(name) => write!(f, "IpAndHeader({:?})", name),
			MurThrottlerKey::Custom(_) => write!(f, "Custom(<function>)"),
			MurThrottlerKey::Global => write!(f, "Global"),
		}
	}
}

impl MurThrottlerKey {
	pub fn extract(&self, ctx: &MurRequestContext) -> Option<String> {
		match self {
			MurThrottlerKey::Ip => Self::extract_ip(ctx),
			MurThrottlerKey::Header(name) => ctx.header(name).map(|s| s.to_string()),
			MurThrottlerKey::BearerToken => Self::extract_bearer_token(ctx),
			MurThrottlerKey::IpAndHeader(name) => {
				let ip = Self::extract_ip(ctx)?;
				let header = ctx.header(name).map(|s| s.to_string()).unwrap_or_default();
				Some(format!("{}:{}", ip, header))
			}
			MurThrottlerKey::Custom(f) => f(ctx),
			MurThrottlerKey::Global => Some("__global__".to_string()),
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
