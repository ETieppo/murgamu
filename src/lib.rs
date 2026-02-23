// Copyright (C) 2026  Emerson Alexandre Tieppo Jr. - Murgamü

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.

// You should have received a copy of the GNU Lesser General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod core;
pub mod server;

pub use chrono;
pub use hyper;
pub use serde;
pub use serde_json;
pub use tokio;

pub use murgamu_macros::MurDto;
pub use murgamu_macros::MurEntity;
pub use murgamu_macros::api;
pub use murgamu_macros::body;
pub use murgamu_macros::controller;
pub use murgamu_macros::delete;
pub use murgamu_macros::get;
pub use murgamu_macros::guard;
pub use murgamu_macros::head;
pub use murgamu_macros::header;
pub use murgamu_macros::html_response;
pub use murgamu_macros::injectable;
pub use murgamu_macros::interceptor;
pub use murgamu_macros::json_response;
pub use murgamu_macros::main;
pub use murgamu_macros::module;
pub use murgamu_macros::no_content_response;
pub use murgamu_macros::ok_response;
pub use murgamu_macros::options;
pub use murgamu_macros::param;
pub use murgamu_macros::patch;
pub use murgamu_macros::post;
pub use murgamu_macros::public;
pub use murgamu_macros::put;
pub use murgamu_macros::query;
pub use murgamu_macros::role;
pub use murgamu_macros::route;
pub use murgamu_macros::service;
pub use murgamu_macros::text_response;
pub use murgamu_macros::validate;

pub use core::utils::MurResponder;

pub use server::MurServer;
pub use server::MurServerRunner;
pub use server::aliases::MurFuture;
pub use server::aliases::MurMainResult;
pub use server::aliases::MurPathParams;
pub use server::aliases::MurQueryParams;
pub use server::aliases::MurReq;
pub use server::aliases::MurRequest;
pub use server::aliases::MurRes;
pub use server::aliases::MurResInfallible;
pub use server::aliases::MurResponse;
pub use server::aliases::MurRouteHandler;
pub use server::config::ConfigMetadata;
pub use server::config::MurConfig;
pub use server::config::MurConfigBuilder;
pub use server::config::MurConfigError;
pub use server::config::MurConfigProvider;
pub use server::config::MurConfigProviderBuilder;
pub use server::config::MurConfigResult;
pub use server::config::MurConfigService;
pub use server::config::MurConfigSource;
pub use server::config::MurEnv;
pub use server::config::MurEnvProfile;
pub use server::config::MurEnvSource;
pub use server::config::MurFileSource;
pub use server::config::MurFromConfig;
pub use server::config::MurMemorySource;
pub use server::config::MurServerConfig;
pub use server::config::mur_current_env;
pub use server::config::mur_env;
pub use server::config::mur_env_is_true;
pub use server::config::mur_env_or;
pub use server::config::mur_env_parse;
pub use server::config::mur_env_parse_or;
pub use server::config::mur_is_development;
pub use server::config::mur_is_production;
pub use server::config::mur_is_test;
pub use server::config::mur_load_config;
pub use server::config::mur_load_config_required;
pub use server::controller::IntoController;
pub use server::controller::MurCloneController;
pub use server::controller::MurController;
pub use server::controller::MurControllerFactory;
pub use server::controller::MurControllers;
pub use server::controller::controllers;
pub use server::decorator::MurDecorator;
pub use server::error::MurError;
pub use server::error::MurExceptionFilter;
pub use server::guard::MurGuard;
pub use server::guard::MurGuardFuture;
pub use server::guard::MurGuardSync;
pub use server::http::MurBody;
pub use server::http::MurExtractor;
pub use server::http::MurExtractorSync;
pub use server::http::MurHeader;
pub use server::http::MurHttpResponse;
pub use server::http::MurIntoResponse;
pub use server::http::MurJson;
pub use server::http::MurMethod;
pub use server::http::MurPath;
pub use server::http::MurQuery;
pub use server::http::MurQueryParam;
pub use server::http::MurRequestContext;
pub use server::http::MurResExt;
pub use server::http::MurResponseBuilder;
pub use server::http::MurText;
pub use server::http::extractors::Param;
pub use server::http::multipart::MurFormField;
pub use server::http::multipart::MurMultipart;
pub use server::http::multipart::MurMultipartConfig;
pub use server::http::multipart::MurUploadedFile;
pub use server::interceptor::MurInterceptor;
pub use server::interceptor::MurInterceptorFuture;
pub use server::middleware::MurMiddleware;
pub use server::middleware::MurNext;
pub use server::middleware::rate_limit::MurRateLimit;
pub use server::middleware::rate_limit::RateLimitAlgorithm;
pub use server::middleware::rate_limit::RateLimitConfig;
pub use server::middleware::rate_limit::RateLimitKey;
pub use server::middleware::timeout::MurTimeout;
pub use server::middleware::timeout::TimeoutConfig;
pub use server::module::MurModule;
pub use server::module::MurModuleConfig;
pub use server::pipe::MurPipe;
pub use server::pipe::MurPipeAsync;
pub use server::provider::MurProvider;
pub use server::provider::MurProviderScope;
pub use server::router::MurRouteBuilder;
pub use server::router::MurRouteDefinition;
pub use server::router::MurRouteInfo;
pub use server::router::MurRoutePattern;
pub use server::router::MurRouter;
pub use server::service::MurDependencies;
pub use server::service::MurInjectable;
pub use server::service::MurInjects;
pub use server::service::MurService;
pub use server::service::MurServiceContainer;
pub use server::service::MurServiceContainerBuilder;
pub use server::service::MurServiceFactory;
pub use server::service::MurServices;

pub mod prelude {
	pub use http;
	pub use http::StatusCode;
	pub use hyper;
	pub use hyper::body::Bytes;
	pub use serde;
	pub use serde::{Deserialize, Serialize};
	pub use serde_json;
	pub use serde_json::json;
	pub use serde_urlencoded;
	pub use std::any::TypeId;
	pub use std::collections::HashMap;
	pub use std::sync::Arc;
	pub use tokio;

	pub use crate::IntoController;
	pub use crate::MurBody;
	pub use crate::MurCloneController;
	pub use crate::MurConfig;
	pub use crate::MurConfigBuilder;
	pub use crate::MurConfigService;
	pub use crate::MurController;
	pub use crate::MurControllers;
	pub use crate::MurDecorator;
	pub use crate::MurDto;
	pub use crate::MurEntity;
	pub use crate::MurEnv;
	pub use crate::MurEnvProfile;
	pub use crate::MurError;
	pub use crate::MurExceptionFilter;
	pub use crate::MurExtractor;
	pub use crate::MurExtractorSync;
	pub use crate::MurFuture;
	pub use crate::MurGuard;
	pub use crate::MurGuardFuture;
	pub use crate::MurGuardSync;
	pub use crate::MurHeader;
	pub use crate::MurHttpResponse;
	pub use crate::MurInjectable;
	pub use crate::MurInjects;
	pub use crate::MurInterceptor;
	pub use crate::MurInterceptorFuture;
	pub use crate::MurIntoResponse;
	pub use crate::MurJson;
	pub use crate::MurMethod;
	pub use crate::MurMiddleware;
	pub use crate::MurModule;
	pub use crate::MurModuleConfig;
	pub use crate::MurNext;
	pub use crate::MurPath;
	pub use crate::MurPathParams;
	pub use crate::MurPipe;
	pub use crate::MurPipeAsync;
	pub use crate::MurProvider;
	pub use crate::MurProviderScope;
	pub use crate::MurQuery;
	pub use crate::MurQueryParam;
	pub use crate::MurQueryParams;
	pub use crate::MurReq;
	pub use crate::MurRequest;
	pub use crate::MurRequestContext;
	pub use crate::MurRes;
	pub use crate::MurResExt;
	pub use crate::MurResInfallible;
	pub use crate::MurResponse;
	pub use crate::MurResponseBuilder;
	pub use crate::MurRouteBuilder;
	pub use crate::MurRouteDefinition;
	pub use crate::MurRouteHandler;
	pub use crate::MurRouteInfo;
	pub use crate::MurRoutePattern;
	pub use crate::MurRouter;
	pub use crate::MurService;
	pub use crate::MurServiceContainer;
	pub use crate::MurServiceFactory;
	pub use crate::api;
	pub use crate::body;
	pub use crate::controller;
	pub use crate::controllers;
	pub use crate::delete;
	pub use crate::get;
	pub use crate::guard;
	pub use crate::head;
	pub use crate::header;
	pub use crate::html_response;
	pub use crate::injectable;
	pub use crate::interceptor;
	pub use crate::json_response;
	pub use crate::module;
	pub use crate::mur_current_env;
	pub use crate::mur_html;
	pub use crate::mur_is_development;
	pub use crate::mur_is_production;
	pub use crate::mur_json;
	pub use crate::mur_load_config;
	pub use crate::mur_text;
	pub use crate::no_content_response;
	pub use crate::ok_response;
	pub use crate::options;
	pub use crate::param;
	pub use crate::patch;
	pub use crate::post;
	pub use crate::public;
	pub use crate::put;
	pub use crate::query;
	pub use crate::role;
	pub use crate::route;
	pub use crate::server::http::sse::MurSse;
	pub use crate::server::http::sse::MurSseChannel;
	pub use crate::server::http::sse::MurSseConfig;
	pub use crate::server::http::sse::MurSseConnection;
	pub use crate::server::http::sse::MurSseEvent;
	pub use crate::server::http::sse::MurSseReceiver;
	pub use crate::server::http::sse::MurSseSendError;
	pub use crate::server::http::sse::MurSseSender;
	pub use crate::server::http::sse::MurSseStream;
	pub use crate::server::http::sse::mur_sse_channel;
	pub use crate::server::http::sse::mur_sse_data;
	pub use crate::server::http::sse::mur_sse_event;
	pub use crate::server::http::sse::mur_sse_headers;
	pub use crate::server::http::sse::mur_sse_json;
	pub use crate::service;
	pub use crate::text_response;
	pub use crate::validate;
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "Murgamü";

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_version() {
		assert!(!VERSION.is_empty());
	}

	#[test]
	fn test_name() {
		assert_eq!(NAME, "Murgamü");
	}
}
