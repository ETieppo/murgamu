use crate::types::{ParamInfo, ParamKind};
use {proc_macro2::TokenStream, quote::quote, syn::Ident};

pub fn generate_handler_code(method_name: &Ident, params: &[ParamInfo]) -> TokenStream {
	let mut extractions = Vec::new();
	let mut call_args = Vec::new();

	for param in params {
		let name = &param.name;
		let extraction = match &param.kind {
			ParamKind::Context => {
				call_args.push(quote!(#name));
				quote! { let #name = ctx.clone(); }
			}
			ParamKind::Json(inner_ty) => {
				call_args.push(quote!(#name));
				quote! {
				  let #name: MurJson<#inner_ty> = match ctx.json() {
					Ok(data) => MurJson(data),
					Err(e) => {
					  return MurResponder::error(&format!("Failed to parse JSON: {}", e));
					}
				  };
				}
			}
			ParamKind::Query(inner_ty) => {
				call_args.push(quote!(#name));
				quote! {
				  let #name: MurQuery<#inner_ty> = {
					let query_string = ctx.parts.uri.query().unwrap_or("");
					match serde_urlencoded::from_str(query_string) {
					  Ok(data) => MurQuery(data),
					  Err(e) => {
						return MurResponder::error(&format!("Failed to parse query: {}", e));
					  }
					}
				  };
				}
			}
			ParamKind::Path(inner_ty) => {
				call_args.push(quote!(#name));
				quote! {
				  let #name: MurPath<#inner_ty> = {
					let json_value = serde_json::to_value(&ctx.path_params)
					  .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
					match serde_json::from_value(json_value) {
					  Ok(data) => MurPath(data),
					  Err(e) => {
						return MurResponder::error(&format!("Failed to parse path: {}", e));
					  }
					}
				  };
				}
			}
			ParamKind::Param(inner_ty) => {
				call_args.push(quote!(#name));
				let param_name_str = name.to_string();
				quote! {
				  let #name: murgamu::Param<#inner_ty> = {
					use std::str::FromStr;
					match ctx.path_params.get(#param_name_str) {
					  Some(value) => {
						match <#inner_ty>::from_str(value) {
						  Ok(parsed) => murgamu::Param(parsed),
						  Err(e) => {
							return MurResponder::error(&format!(
							  "Failed to parse parameter '{}': {}", #param_name_str, e
							));
						  }
						}
					  }
					  None => {
						return MurResponder::error(&format!(
						  "Missing path parameter: {}", #param_name_str
						));
					  }
					}
				  };
				}
			}
			// ParamKind::Inject(inner_ty) => {
			// 	call_args.push(quote!(#name));
			// 	quote! {
			// 	  let #name: murgamu::Inject<#inner_ty> = match ctx.container.get::<#inner_ty>() {
			// 		Some(service) => murgamu::Inject::new(service),
			// 		None => {
			// 		  return MurResponder::error(&format!(
			// 			"Service not found: {}", stringify!(#inner_ty)
			// 		  ));
			// 		}
			// 	  };
			// 	}
			// }
			ParamKind::Header => {
				call_args.push(quote!(#name));
				quote! { let #name = MurHeader(String::new()); }
			}
			ParamKind::Body => {
				call_args.push(quote!(#name));
				quote! {
				  let #name = match &ctx.body {
					Some(bytes) => MurBody(bytes.clone()),
					None => return MurResponder::error("No body provided"),
				  };
				}
			}
			ParamKind::Text => {
				call_args.push(quote!(#name));
				quote! {
				  let #name = match &ctx.body {
					Some(bytes) => match String::from_utf8(bytes.to_vec()) {
					  Ok(s) => MurText(s),
					  Err(e) => return MurResponder::error(&format!("Invalid UTF-8: {}", e)),
					},
					None => return MurResponder::error("No body provided"),
				  };
				}
			}
			ParamKind::Parts => {
				call_args.push(quote!(#name));
				quote! { let #name = (*ctx.parts).clone(); }
			}
			ParamKind::RawBody => {
				call_args.push(quote!(#name));
				quote! { let #name = ctx.body.clone(); }
			}
			ParamKind::Container => {
				call_args.push(quote!(#name));
				quote! { let #name = ctx.container.clone(); }
			}
			ParamKind::CustomJson(ty) => {
				call_args.push(quote!(#name));
				quote! {
				  let #name: #ty = match ctx.json() {
					Ok(data) => data,
					Err(e) => return MurResponder::error(&format!("Failed to parse body: {}", e)),
				  };
				}
			}
			ParamKind::SelfRef | ParamKind::Unknown => continue,
		};
		extractions.push(extraction);
	}

	quote! {
		 {
		   let controller_clone = controller.clone();
		   Arc::new(move |ctx: MurRequestContext| -> MurFuture {
	   let controller = controller_clone.clone();
	   Box::pin(async move {
		 #(#extractions)*
		 controller.#method_name(#(#call_args),*).await
	   })
	   })
	as MurRouteHandler
		 }
	   }
}
