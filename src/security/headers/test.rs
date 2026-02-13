use crate::security::headers::{
	content_security_policy::ContentSecurityPolicy, cross_domain_policy::CrossDomainPolicy,
	cross_origin_embedder_policy::CrossOriginEmbedderPolicy,
	cross_origin_opener_policy::CrossOriginOpenerPolicy,
	cross_origin_resource_policy::CrossOriginResourcePolicy, hsts_config::HstsConfig,
	permission_policy::PermissionsPolicy, referrer_policy::ReferrerPolicy,
	security_headers::MurSecurityHeaders, security_headers_config::SecurityHeadersConfig,
	x_frame_options::XFrameOptions, xss_protection::XssProtection,
};

#[test]
fn test_config_defaults() {
	let config = SecurityHeadersConfig::default();
	assert!(config.content_security_policy.is_some());
	assert!(config.x_frame_options.is_some());
	assert!(config.x_content_type_options);
	assert!(config.hsts.is_some());
}

#[test]
fn test_x_frame_options() {
	assert_eq!(XFrameOptions::Deny.as_str(), "DENY");
	assert_eq!(XFrameOptions::SameOrigin.as_str(), "SAMEORIGIN");
}

#[test]
fn test_xss_protection() {
	assert_eq!(XssProtection::Disabled.as_str(), "0");
	assert_eq!(XssProtection::Enabled.as_str(), "1");
	assert_eq!(XssProtection::EnabledBlock.as_str(), "1; mode=block");
}

#[test]
fn test_hsts_config() {
	let hsts = HstsConfig::new();
	assert_eq!(hsts.max_age, 15552000);
	assert!(!hsts.include_subdomains);
	assert!(!hsts.preload);

	let hsts = HstsConfig::new()
		.max_age(31536000)
		.include_subdomains()
		.preload();
	assert_eq!(hsts.max_age, 31536000);
	assert!(hsts.include_subdomains);
	assert!(hsts.preload);
	assert_eq!(
		hsts.to_header_value(),
		"max-age=31536000; includeSubDomains; preload"
	);
}

#[test]
fn test_hsts_one_year() {
	let hsts = HstsConfig::new().one_year();
	assert_eq!(hsts.max_age, 31536000);
}

#[test]
fn test_referrer_policy() {
	assert_eq!(ReferrerPolicy::NoReferrer.as_str(), "no-referrer");
	assert_eq!(
		ReferrerPolicy::StrictOriginWhenCrossOrigin.as_str(),
		"strict-origin-when-cross-origin"
	);
}

#[test]
fn test_content_security_policy_builder() {
	let csp = ContentSecurityPolicy::new()
		.default_src("'self'")
		.script_src("'self' 'unsafe-inline'")
		.style_src("'self'")
		.img_src("'self' data:");

	let value = csp.build();
	assert!(value.contains("default-src 'self'"));
	assert!(value.contains("script-src 'self' 'unsafe-inline'"));
	assert!(value.contains("style-src 'self'"));
	assert!(value.contains("img-src 'self' data:"));
}

#[test]
fn test_csp_report_only() {
	let csp = ContentSecurityPolicy::new().report_only();
	assert_eq!(csp.header_name(), "Content-Security-Policy-Report-Only");

	let csp = ContentSecurityPolicy::new();
	assert_eq!(csp.header_name(), "Content-Security-Policy");
}

#[test]
fn test_csp_upgrade_insecure_requests() {
	let csp = ContentSecurityPolicy::new()
		.default_src("'self'")
		.upgrade_insecure_requests();

	let value = csp.build();
	assert!(value.contains("upgrade-insecure-requests"));
}

#[test]
fn test_permissions_policy_builder() {
	let policy = PermissionsPolicy::new()
		.deny("geolocation")
		.allow_self("camera")
		.allow_all("autoplay");

	let value = policy.build();
	assert!(value.contains("geolocation=()"));
	assert!(value.contains("camera=(self)"));
	assert!(value.contains("autoplay=*"));
}

#[test]
fn test_permissions_policy_disable_all() {
	let policy = PermissionsPolicy::new().disable_all();
	let value = policy.build();
	assert!(value.contains("camera=()"));
	assert!(value.contains("microphone=()"));
	assert!(value.contains("geolocation=()"));
}

#[test]
fn test_cross_origin_policies() {
	assert_eq!(
		CrossOriginEmbedderPolicy::UnsafeNone.as_str(),
		"unsafe-none"
	);
	assert_eq!(
		CrossOriginEmbedderPolicy::RequireCorp.as_str(),
		"require-corp"
	);

	assert_eq!(CrossOriginOpenerPolicy::SameOrigin.as_str(), "same-origin");
	assert_eq!(
		CrossOriginOpenerPolicy::SameOriginAllowPopups.as_str(),
		"same-origin-allow-popups"
	);

	assert_eq!(
		CrossOriginResourcePolicy::CrossOrigin.as_str(),
		"cross-origin"
	);
	assert_eq!(CrossOriginResourcePolicy::SameSite.as_str(), "same-site");
}

#[test]
fn test_security_headers_builder() {
	let headers = MurSecurityHeaders::new()
		.content_security_policy("default-src 'self'")
		.x_frame_options(XFrameOptions::Deny)
		.hsts(HstsConfig::new().one_year())
		.referrer_policy(ReferrerPolicy::NoReferrer);

	assert_eq!(
		headers.config.content_security_policy,
		Some("default-src 'self'".to_string())
	);
	assert_eq!(headers.config.x_frame_options, Some(XFrameOptions::Deny));
}

#[test]
fn test_security_headers_disable() {
	let headers = MurSecurityHeaders::new()
		.no_csp()
		.no_hsts()
		.no_x_frame_options();

	assert!(headers.config.content_security_policy.is_none());
	assert!(headers.config.hsts.is_none());
	assert!(headers.config.x_frame_options.is_none());
}

#[test]
fn test_custom_headers() {
	let headers = MurSecurityHeaders::new()
		.custom_header("X-Custom-Header", "custom-value")
		.remove_header("Server");

	assert_eq!(
		headers.config.custom_headers.get("X-Custom-Header"),
		Some(&"custom-value".to_string())
	);
	assert!(headers
		.config
		.remove_headers
		.contains(&"Server".to_string()));
}

#[test]
fn test_convenience_functions() {
	let _default = MurSecurityHeaders::default();
	let strict = MurSecurityHeaders::new()
        .content_security_policy("default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; object-src 'none'; frame-ancestors 'none'")
        .x_frame_options(XFrameOptions::Deny)
        .hsts(HstsConfig::new().one_year().include_subdomains())
        .referrer_policy(ReferrerPolicy::NoReferrer)
        .permissions(PermissionsPolicy::new().disable_all());
	let api = MurSecurityHeaders::new()
		.no_csp()
		.x_frame_options(XFrameOptions::Deny)
		.hsts(HstsConfig::new().one_year())
		.referrer_policy(ReferrerPolicy::NoReferrer)
		.cross_origin_resource_policy(CrossOriginResourcePolicy::SameOrigin);

	assert_eq!(strict.config.x_frame_options, Some(XFrameOptions::Deny));
	assert!(api.config.content_security_policy.is_none());
}

#[test]
fn test_cross_domain_policy() {
	assert_eq!(CrossDomainPolicy::None.as_str(), "none");
	assert_eq!(CrossDomainPolicy::MasterOnly.as_str(), "master-only");
	assert_eq!(CrossDomainPolicy::All.as_str(), "all");
}

#[test]
fn test_security_headers_clone() {
	let headers = MurSecurityHeaders::new().content_security_policy("default-src 'self'");
	let cloned = headers.clone();

	assert_eq!(
		cloned.config.content_security_policy,
		headers.config.content_security_policy
	);
}
