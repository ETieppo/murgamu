use std::collections::HashMap;

use super::{
	cross_domain_policy::CrossDomainPolicy,
	cross_origin_embedder_policy::CrossOriginEmbedderPolicy,
	cross_origin_opener_policy::CrossOriginOpenerPolicy,
	cross_origin_resource_policy::CrossOriginResourcePolicy, hsts_config::HstsConfig,
	referrer_policy::ReferrerPolicy, x_frame_options::XFrameOptions, xss_protection::XssProtection,
};

#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
	pub content_security_policy: Option<String>,
	pub x_frame_options: Option<XFrameOptions>,
	pub x_content_type_options: bool,
	pub x_xss_protection: Option<XssProtection>,
	pub hsts: Option<HstsConfig>,
	pub referrer_policy: Option<ReferrerPolicy>,
	pub permissions_policy: Option<String>,
	pub x_dns_prefetch_control: Option<bool>,
	pub x_permitted_cross_domain_policies: Option<CrossDomainPolicy>,
	pub x_download_options: bool,
	pub cross_origin_embedder_policy: Option<CrossOriginEmbedderPolicy>,
	pub cross_origin_opener_policy: Option<CrossOriginOpenerPolicy>,
	pub cross_origin_resource_policy: Option<CrossOriginResourcePolicy>,
	pub origin_agent_cluster: bool,
	pub custom_headers: HashMap<String, String>,
	pub remove_headers: Vec<String>,
}

impl Default for SecurityHeadersConfig {
	fn default() -> Self {
		Self {
			content_security_policy: Some("default-src 'self'".to_string()),
			x_frame_options: Some(XFrameOptions::SameOrigin),
			x_content_type_options: true,
			x_xss_protection: Some(XssProtection::Disabled),
			hsts: Some(HstsConfig::default()),
			referrer_policy: Some(ReferrerPolicy::StrictOriginWhenCrossOrigin),
			permissions_policy: None,
			x_dns_prefetch_control: Some(false),
			x_permitted_cross_domain_policies: Some(CrossDomainPolicy::None),
			x_download_options: true,
			cross_origin_embedder_policy: None,
			cross_origin_opener_policy: Some(CrossOriginOpenerPolicy::SameOrigin),
			cross_origin_resource_policy: Some(CrossOriginResourcePolicy::SameOrigin),
			origin_agent_cluster: true,
			custom_headers: HashMap::new(),
			remove_headers: vec!["X-Powered-By".to_string(), "Server".to_string()],
		}
	}
}
