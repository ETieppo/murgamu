use crate::controller::generate_handler_code;
use crate::controller::get_base_path::get_base_path;
use crate::core::{
	analyze_parameter, gen_constructor, has_self, implments_impl_mur_dependencies, is_constructor,
	normalize_path,
};
use crate::types::ParamInfo;
use proc_macro2::TokenStream;
use quote::quote;
use syn::fold::{self, Fold};
use syn::{FnArg, ItemImpl, Meta, MetaList};

struct StripUsePipeAttrs;

impl Fold for StripUsePipeAttrs {
	fn fold_fn_arg(&mut self, mut arg: FnArg) -> FnArg {
		if let FnArg::Typed(ref mut pat_type) = arg {
			pat_type.attrs.retain(|attr| {
				!attr.path().is_ident("use_pipe")
					&& !attr.path().is_ident("body")
					&& !attr.path().is_ident("param")
			});
		}
		fold::fold_fn_arg(self, arg)
	}
}

pub fn controller_impl(args: proc_macro::TokenStream, input: ItemImpl) -> TokenStream {
	let base_path = get_base_path(args);
	let impl_type = &input.self_ty;
	let generics = &input.generics;
	let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();
	let mut route_registrations: Vec<TokenStream> = Vec::new();

	for item in &input.items {
		let syn::ImplItem::Fn(method) = item else {
			continue;
		};
		let method_name = &method.sig.ident;
		let method_inputs = &method.sig.inputs;
		let mut is_public = false;
		let mut allowed_roles: Vec<syn::Path> = Vec::new();

		if !is_constructor(&method.sig.output) && !has_self(method_inputs) {
			let method_name_str = method_name.to_string();
			return quote! {
				compile_error!(concat!(
					"Invalid controller method `",
					#method_name_str,
					"`.\n\n",
					"Route handlers in Murgamu must be instance methods and ",
					"receive `&self` as the first parameter.\n\n",
				));
			};
		}

		for attr in &method.attrs {
			if attr.path().is_ident("public") {
				is_public = true;
			}

			if attr.path().is_ident("role") {
				let parsed: syn::punctuated::Punctuated<syn::Path, syn::Token![,]> = attr
					.parse_args_with(syn::punctuated::Punctuated::parse_terminated)
					.unwrap();

				for path in parsed {
					allowed_roles.push(path);
				}
			}

			let Meta::List(MetaList { path, tokens, .. }) = &attr.meta else {
				continue;
			};

			let Some(ident) = path.get_ident() else {
				continue;
			};

			let http_method = ident.to_string();
			if !matches!(
				http_method.as_str(),
				"get" | "post" | "put" | "delete" | "patch" | "head" | "options"
			) {
				continue;
			}

			let http_method_upper = http_method.to_uppercase();
			let tokens_str = tokens.to_string();
			let route_path = normalize_path(tokens_str.trim_matches('"'));
			let full_path = merge_paths(&base_path, &route_path);
			let params: Vec<ParamInfo> = method_inputs
				.iter()
				.filter_map(|arg| {
					if let FnArg::Typed(pat_type) = arg {
						Some(analyze_parameter(pat_type))
					} else {
						None
					}
				})
				.collect();
			let handler_code = generate_handler_code(method_name, &params);

			route_registrations.push(quote! {
				routes.push(murgamu::MurRouteDefinition {
					method: #http_method_upper.to_string(),
					path: #full_path.to_string(),
					handler: #handler_code,
					is_public: #is_public,
					allowed_roles: vec![#(stringify!(#allowed_roles).to_string()),*],
				});
			});
		}
	}

	let (ensure_constructor, ctor_lets, ctor_args, required_container_typeids) =
		gen_constructor(&input);

	let controller_trait_impl = quote! {
		impl #impl_generics murgamu::MurController for #impl_type #where_clause {
			fn routes(
				self: std::sync::Arc<Self>,
				container: &murgamu::MurServiceContainer,
			) -> Vec<murgamu::MurRouteDefinition> {
				use std::sync::Arc;
				use murgamu::{
					MurServiceContainer,
					MurRouteHandler,
					MurFuture,
					MurRes,
					MurRequestContext,
					MurJson,
					MurQuery,
					MurPath,
					Param,
					MurError,
					MurHttpResponse,
					MurResponder,
					MurRouteDefinition,
				};

				let controller = self;
				let _ = &container;
				let mut routes: Vec<MurRouteDefinition> = Vec::new();

				#(#route_registrations)*
				routes
			}

			fn base_path(&self) -> &str {
				#base_path
			}
		}
	};

	let controller_factory_impl = quote! {
		impl #impl_generics murgamu::MurControllerFactory for #impl_type #where_clause {
			fn __create_factory(injects: &murgamu::MurInjects, _container: &murgamu::MurServiceContainer) -> Self {
				#(#ctor_lets)*
				Self::new(#(#ctor_args),*)
			}
		}
	};

	let deps_impl = implments_impl_mur_dependencies(
		required_container_typeids,
		impl_generics,
		impl_type,
		where_clause,
	);

	let clean_input = StripUsePipeAttrs.fold_item_impl(input);

	quote! {
		#clean_input
		#ensure_constructor
		#controller_trait_impl
		#controller_factory_impl
		#deps_impl
	}
}

fn merge_paths(base: &str, route: &str) -> String {
	match (base, route) {
		("/", r) => r.to_string(),
		(b, "/") => b.to_string(),
		(b, r) => format!("{}{}", b, r),
	}
}
