use {
	quote::quote,
	syn::{Ident, Pat, PatType},
};

use super::{extract_generic_type, ParamInfo, ParamKind};

pub fn analyze_parameter(pat_type: &PatType) -> ParamInfo {
	let name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
		pat_ident.ident.clone()
	} else {
		Ident::new("_param", proc_macro2::Span::call_site())
	};

	let ty = &pat_type.ty;
	let ty_str = quote!(#ty).to_string().replace(" ", "");
	let ty_tokens = quote!(#ty);
	let is_optional = ty_str.starts_with("Option<");

	let kind = if ty_str.contains("MurRequestContext") || ty_str.contains("MurReq") {
		ParamKind::Context
	} else if ty_str.starts_with("MurJson<") || ty_str.starts_with("murgamu::MurJson<") {
		let inner = extract_generic_type(&ty_str, "MurJson");
		ParamKind::Json(inner.parse().unwrap_or(quote!(serde_json::Value)))
	} else if ty_str.starts_with("MurQuery<") || ty_str.starts_with("murgamu::MurQuery<") {
		let inner = extract_generic_type(&ty_str, "MurQuery");
		ParamKind::Query(
			inner
				.parse()
				.unwrap_or(quote!(std::collections::HashMap<String, String>)),
		)
	} else if ty_str.starts_with("MurPath<") || ty_str.starts_with("murgamu::MurPath<") {
		let inner = extract_generic_type(&ty_str, "MurPath");
		ParamKind::Path(inner.parse().unwrap_or(quote!(String)))
	} else if ty_str.starts_with("Param<") || ty_str.starts_with("murgamu::Param<") {
		let inner = extract_generic_type(&ty_str, "Param");
		ParamKind::Param(inner.parse().unwrap_or(quote!(String)))
	}
	// else if ty_str.starts_with("Inject<")
	//   || ty_str.starts_with("murgamu::Inject<")
	// {
	//   let inner = extract_generic_type(&ty_str, "Inject");
	//   ParamKind::Inject(inner.parse().unwrap_or(quote!(())))
	// }
	else if ty_str.contains("MurHeader") {
		ParamKind::Header
	} else if ty_str.contains("MurBody") {
		ParamKind::Body
	} else if ty_str.contains("MurText") {
		ParamKind::Text
	} else if ty_str.contains("Parts") && ty_str.contains("request") {
		ParamKind::Parts
	} else if ty_str.contains("Option<Bytes>") || ty_str.contains("Option<hyper::body::Bytes>") {
		ParamKind::RawBody
	} else if ty_str.contains("Arc<MurServiceContainer>") || ty_str.contains("MurServiceContainer")
	{
		ParamKind::Container
	} else if ty_str.contains("Bytes") {
		ParamKind::RawBody
	} else {
		ParamKind::CustomJson(ty_tokens.clone())
	};

	ParamInfo {
		name,
		kind,
		ty: ty_tokens,
		is_optional,
	}
}
