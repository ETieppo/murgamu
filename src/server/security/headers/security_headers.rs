use super::{
	content_security_policy::ContentSecurityPolicy, cross_domain_policy::CrossDomainPolicy,
	cross_origin_embedder_policy::CrossOriginEmbedderPolicy,
	cross_origin_opener_policy::CrossOriginOpenerPolicy,
	cross_origin_resource_policy::CrossOriginResourcePolicy, hsts_config::HstsConfig,
	permission::PermissionsPolicy, referrer_policy::ReferrerPolicy,
	security_headers_config::SecurityHeadersConfig, x_frame_options::XFrameOptions,
	xss_protection::XssProtection,
};
use crate::server::aliases::MurFuture;
use crate::server::http::MurRequestContext;
use crate::server::middleware::{MurMiddleware, MurNext};

pub struct MurSecurityHeaders {
	pub config: SecurityHeadersConfig,
}

impl MurSecurityHeaders {
	pub fn new() -> Self {
		Self {
			config: SecurityHeadersConfig::default(),
		}
	}

	pub fn from_config(config: SecurityHeadersConfig) -> Self {
		Self { config }
	}

	pub fn content_security_policy(mut self, value: impl Into<String>) -> Self {
		self.config.content_security_policy = Some(value.into());
		self
	}

	pub fn csp(mut self, csp: ContentSecurityPolicy) -> Self {
		self.config.content_security_policy = Some(csp.build());
		self
	}

	pub fn no_csp(mut self) -> Self {
		self.config.content_security_policy = None;
		self
	}

	pub fn x_frame_options(mut self, value: XFrameOptions) -> Self {
		self.config.x_frame_options = Some(value);
		self
	}

	pub fn no_x_frame_options(mut self) -> Self {
		self.config.x_frame_options = None;
		self
	}

	pub fn x_content_type_options(mut self, enable: bool) -> Self {
		self.config.x_content_type_options = enable;
		self
	}

	pub fn x_xss_protection(mut self, value: XssProtection) -> Self {
		self.config.x_xss_protection = Some(value);
		self
	}

	pub fn no_x_xss_protection(mut self) -> Self {
		self.config.x_xss_protection = None;
		self
	}

	pub fn hsts(mut self, config: HstsConfig) -> Self {
		self.config.hsts = Some(config);
		self
	}

	pub fn no_hsts(mut self) -> Self {
		self.config.hsts = None;
		self
	}

	pub fn referrer_policy(mut self, value: ReferrerPolicy) -> Self {
		self.config.referrer_policy = Some(value);
		self
	}

	pub fn no_referrer_policy(mut self) -> Self {
		self.config.referrer_policy = None;
		self
	}

	pub fn permissions_policy(mut self, value: impl Into<String>) -> Self {
		self.config.permissions_policy = Some(value.into());
		self
	}

	pub fn permissions(mut self, policy: PermissionsPolicy) -> Self {
		self.config.permissions_policy = Some(policy.build());
		self
	}

	pub fn no_permissions_policy(mut self) -> Self {
		self.config.permissions_policy = None;
		self
	}

	pub fn x_dns_prefetch_control(mut self, enable: bool) -> Self {
		self.config.x_dns_prefetch_control = Some(enable);
		self
	}

	pub fn x_permitted_cross_domain_policies(mut self, value: CrossDomainPolicy) -> Self {
		self.config.x_permitted_cross_domain_policies = Some(value);
		self
	}

	pub fn cross_origin_embedder_policy(mut self, value: CrossOriginEmbedderPolicy) -> Self {
		self.config.cross_origin_embedder_policy = Some(value);
		self
	}

	pub fn cross_origin_opener_policy(mut self, value: CrossOriginOpenerPolicy) -> Self {
		self.config.cross_origin_opener_policy = Some(value);
		self
	}

	pub fn cross_origin_resource_policy(mut self, value: CrossOriginResourcePolicy) -> Self {
		self.config.cross_origin_resource_policy = Some(value);
		self
	}

	pub fn origin_agent_cluster(mut self, enable: bool) -> Self {
		self.config.origin_agent_cluster = enable;
		self
	}

	pub fn custom_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
		self.config.custom_headers.insert(name.into(), value.into());
		self
	}

	pub fn remove_header(mut self, name: impl Into<String>) -> Self {
		self.config.remove_headers.push(name.into());
		self
	}

	pub fn apply_headers(
		&self,
		response: &mut http::Response<http_body_util::Full<hyper::body::Bytes>>,
	) {
		let headers = response.headers_mut();

		if let Some(ref csp) = self.config.content_security_policy {
			headers.insert("Content-Security-Policy", csp.parse().unwrap());
		}

		if let Some(ref xfo) = self.config.x_frame_options {
			headers.insert("X-Frame-Options", xfo.as_str().parse().unwrap());
		}

		if self.config.x_content_type_options {
			headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
		}

		if let Some(ref xss) = self.config.x_xss_protection {
			headers.insert("X-XSS-Protection", xss.as_str().parse().unwrap());
		}

		if let Some(ref hsts) = self.config.hsts {
			headers.insert(
				"Strict-Transport-Security",
				hsts.to_header_value().parse().unwrap(),
			);
		}

		if let Some(ref rp) = self.config.referrer_policy {
			headers.insert("Referrer-Policy", rp.as_str().parse().unwrap());
		}

		if let Some(ref pp) = self.config.permissions_policy {
			headers.insert("Permissions-Policy", pp.parse().unwrap());
		}

		if let Some(dns) = self.config.x_dns_prefetch_control {
			let value = if dns { "on" } else { "off" };
			headers.insert("X-DNS-Prefetch-Control", value.parse().unwrap());
		}

		if let Some(ref cdp) = self.config.x_permitted_cross_domain_policies {
			headers.insert(
				"X-Permitted-Cross-Domain-Policies",
				cdp.as_str().parse().unwrap(),
			);
		}

		if self.config.x_download_options {
			headers.insert("X-Download-Options", "noopen".parse().unwrap());
		}

		if let Some(ref coep) = self.config.cross_origin_embedder_policy {
			headers.insert(
				"Cross-Origin-Embedder-Policy",
				coep.as_str().parse().unwrap(),
			);
		}

		if let Some(ref coop) = self.config.cross_origin_opener_policy {
			headers.insert("Cross-Origin-Opener-Policy", coop.as_str().parse().unwrap());
		}

		if let Some(ref corp) = self.config.cross_origin_resource_policy {
			headers.insert(
				"Cross-Origin-Resource-Policy",
				corp.as_str().parse().unwrap(),
			);
		}

		if self.config.origin_agent_cluster {
			headers.insert("Origin-Agent-Cluster", "?1".parse().unwrap());
		}

		for (name, value) in &self.config.custom_headers {
			if let (Ok(n), Ok(v)) = (
				name.parse::<http::header::HeaderName>(),
				value.parse::<http::header::HeaderValue>(),
			) {
				headers.insert(n, v);
			}
		}

		for name in &self.config.remove_headers {
			if let Ok(n) = name.parse::<http::header::HeaderName>() {
				headers.remove(n);
			}
		}
	}
}

impl Default for MurSecurityHeaders {
	fn default() -> Self {
		Self::new()
	}
}

impl Clone for MurSecurityHeaders {
	fn clone(&self) -> Self {
		Self {
			config: self.config.clone(),
		}
	}
}

impl std::fmt::Debug for MurSecurityHeaders {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MurSecurityHeaders")
			.field("config", &self.config)
			.finish()
	}
}

impl MurMiddleware for MurSecurityHeaders {
	fn handle(&self, ctx: MurRequestContext, next: MurNext) -> MurFuture {
		let config = self.config.clone();

		Box::pin(async move {
			let result = next.run(ctx).await;

			match result {
				Ok(mut response) => {
					let headers = response.headers_mut();

					if let Some(ref csp) = config.content_security_policy
						&& let Ok(v) = csp.parse()
					{
						headers.insert("Content-Security-Policy", v);
					}

					if let Some(ref xfo) = config.x_frame_options {
						headers.insert("X-Frame-Options", xfo.as_str().parse().unwrap());
					}

					if config.x_content_type_options {
						headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
					}

					if let Some(ref xss) = config.x_xss_protection {
						headers.insert("X-XSS-Protection", xss.as_str().parse().unwrap());
					}

					if let Some(ref hsts) = config.hsts
						&& let Ok(v) = hsts.to_header_value().parse()
					{
						headers.insert("Strict-Transport-Security", v);
					}

					if let Some(ref rp) = config.referrer_policy {
						headers.insert("Referrer-Policy", rp.as_str().parse().unwrap());
					}

					if let Some(ref pp) = config.permissions_policy
						&& let Ok(v) = pp.parse()
					{
						headers.insert("Permissions-Policy", v);
					}

					if let Some(dns) = config.x_dns_prefetch_control {
						let value = if dns { "on" } else { "off" };
						headers.insert("X-DNS-Prefetch-Control", value.parse().unwrap());
					}

					if let Some(ref cdp) = config.x_permitted_cross_domain_policies {
						headers.insert(
							"X-Permitted-Cross-Domain-Policies",
							cdp.as_str().parse().unwrap(),
						);
					}

					if config.x_download_options {
						headers.insert("X-Download-Options", "noopen".parse().unwrap());
					}

					if let Some(ref coep) = config.cross_origin_embedder_policy {
						headers.insert(
							"Cross-Origin-Embedder-Policy",
							coep.as_str().parse().unwrap(),
						);
					}

					if let Some(ref coop) = config.cross_origin_opener_policy {
						headers
							.insert("Cross-Origin-Opener-Policy", coop.as_str().parse().unwrap());
					}

					if let Some(ref corp) = config.cross_origin_resource_policy {
						headers.insert(
							"Cross-Origin-Resource-Policy",
							corp.as_str().parse().unwrap(),
						);
					}

					if config.origin_agent_cluster {
						headers.insert("Origin-Agent-Cluster", "?1".parse().unwrap());
					}

					for (name, value) in &config.custom_headers {
						if let (Ok(n), Ok(v)) = (
							name.parse::<http::header::HeaderName>(),
							value.parse::<http::header::HeaderValue>(),
						) {
							headers.insert(n, v);
						}
					}

					for name in &config.remove_headers {
						if let Ok(n) = name.parse::<http::header::HeaderName>() {
							headers.remove(n);
						}
					}

					Ok(response)
				}
				Err(e) => Err(e),
			}
		})
	}

	fn name(&self) -> &str {
		"MurSecurityHeaders"
	}
}
