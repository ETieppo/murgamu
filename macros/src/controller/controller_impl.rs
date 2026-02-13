use {
	proc_macro2::TokenStream,
	quote::quote,
	syn::{FnArg, ItemImpl, Lit, Meta, MetaList},
};

use crate::types::ParamInfo;
use crate::utils::{analyze_parameter, is_constructor, normalize_path};

pub fn controller_impl(args: proc_macro::TokenStream, input: ItemImpl) -> TokenStream {
	let base_path = if !args.is_empty() {
		let lit: Lit = syn::parse(args).unwrap_or(Lit::Str(syn::LitStr::new(
			"/",
			proc_macro2::Span::call_site(),
		)));
		if let Lit::Str(lit_str) = lit {
			normalize_path(&lit_str.value())
		} else {
			String::from("/")
		}
	} else {
		String::from("/")
	};

	let impl_type = &input.self_ty;
	let impl_generics = &input.generics;
	let mut route_registrations = Vec::new();

	for item in &input.items {
		if let syn::ImplItem::Fn(method) = item {
			let method_name = &method.sig.ident;
			let method_inputs = &method.sig.inputs;

			if let syn::ImplItem::Fn(method) = item {
				let method_name = &method.sig.ident;
				let method_inputs = &method.sig.inputs;
				let is_constructor = is_constructor(&method.sig.output);

				if !is_constructor {
					let has_self = method_inputs
						.first()
						.map_or(false, |input| matches!(input, FnArg::Receiver(_)));

					if !has_self {
						let method_name_str = method_name.to_string();
						return quote! {
						  compile_error!(concat!(
							"Invalid controller method `",
							#method_name_str,
							"`.\n\n",
							"Route handlers in Murgamü must be instance methods and ",
							"receive `&self` as the first parameter.\n\n",
						  ));
						}
						.into();
					}
				}
			}

			for attr in &method.attrs {
				if let Meta::List(MetaList { path, tokens, .. }) = &attr.meta {
					if let Some(ident) = path.get_ident() {
						let http_method = ident.to_string();

						if matches!(
							http_method.as_str(),
							"get" | "post" | "put" | "delete" | "patch" | "head" | "options"
						) {
							let tokens_str = tokens.to_string();
							let route_path = normalize_path(tokens_str.trim_matches('"'));
							let full_path = if base_path == "/" {
								route_path
							} else if route_path == "/" {
								base_path.clone()
							} else {
								format!("{}{}", base_path, route_path)
							};
							let http_method_upper = http_method.to_uppercase();
							let mut params: Vec<ParamInfo> = Vec::new();

							for input in method_inputs.iter() {
								match input {
									FnArg::Typed(pat_type) => {
										params.push(analyze_parameter(pat_type));
									}
									_ => continue,
								}
							}

							let handler_code = super::generate_handler_code(method_name, &params);

							route_registrations.push(quote! {
							  routes.push((
								#http_method_upper.to_string(),
								#full_path.to_string(),
								#handler_code
							  ));
							});
						}
					}
				}
			}
		}
	}

	let controller_impl = quote! {
	impl #impl_generics MurController for #impl_type {
	  fn routes(
		self: Arc<Self>,
		_container: std::sync::Arc<MurServiceContainer>,
	  ) -> Vec<(String, String, MurRouteHandler)> {
		use murgamu::{
		  MurController, MurServiceContainer, MurRouteHandler, MurFuture, MurRes,
		  MurRequestContext, MurJson, MurQuery, MurPath, Param, MurError, MurHttpResponse,
		  MurResponder
		};

		let mut routes: Vec<(String, String, MurRouteHandler)> = Vec::new();
		let controller = self.clone();

		#(#route_registrations)*

		routes
	  }

	  fn base_path(&self) -> &str {
		#base_path
	  }
	}};

	quote! {
	  #input
	  #controller_impl
	}
}
