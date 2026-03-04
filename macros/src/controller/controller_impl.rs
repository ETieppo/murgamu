use crate::controller::generate_handler_code;
use crate::controller::get_base_path::get_base_path;
use crate::core::analyze_parameter;
use crate::core::gen_constructor;
use crate::core::has_self;
use crate::core::implments_impl_mur_dependencies;
use crate::core::is_constructor;
use crate::core::normalize_path;
use crate::types::ParamInfo;
use proc_macro2::TokenStream;
use quote::quote;
use syn::FnArg;
use syn::ItemImpl;
use syn::Meta;
use syn::MetaList;

pub fn controller_impl(
	args: proc_macro::TokenStream,
	input: ItemImpl,
) -> TokenStream {
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
		let mut allowed_roles: Vec<String> = Vec::new();

		if !is_constructor(&method.sig.output) && !has_self(method_inputs) {
			let method_name_str = method_name.to_string();
			return quote! {
				compile_error!(concat!(
					"Invalid controller method `",
					#method_name_str,
					"`.\n\n",
					"Route handlers in Murgamü must be instance methods and ",
					"receive `&self` as the first parameter.\n\n",
				));
			};
		}

		for attr in &method.attrs {
			if attr.path().is_ident("public") {
				is_public = true;
			}

			if attr.path().is_ident("role")
				&& let Meta::List(meta_list) = &attr.meta
			{
				let tokens_str = meta_list.tokens.to_string();
				for part in tokens_str.split(',') {
					let role = part.trim();
					if let Some(name) = role.split("::").last() {
						allowed_roles.push(name.to_string());
					}
				}
			}

			if let Meta::List(MetaList { path, tokens, .. }) = &attr.meta {
				let Some(ident) = path.get_ident() else {
					continue;
				};
				let http_method = ident.to_string();

				if !matches!(
					http_method.as_str(),
					"get"
						| "post" | "put" | "delete"
						| "patch" | "head" | "options"
				) {
					continue;
				}

				let mut params: Vec<ParamInfo> = Vec::new();
				let http_method_upper = http_method.to_uppercase();
				let tokens_str = tokens.to_string();
				let route_path = normalize_path(tokens_str.trim_matches('"'));
				let full_path = if base_path == "/" {
					route_path
				} else if route_path == "/" {
					base_path.clone()
				} else {
					format!("{}{}", base_path, route_path)
				};

				for input_arg in method_inputs.iter() {
					if let FnArg::Typed(pat_type) = input_arg {
						params.push(analyze_parameter(pat_type));
					}
				}

				let handler_code = generate_handler_code(method_name, &params);

				route_registrations.push(quote! {
					routes.push(murgamu::MurRouteDefinition {
						method: #http_method_upper.to_string(),
						path: #full_path.to_string(),
						handler: #handler_code,
						is_public: #is_public,
						allowed_roles: vec![#(#allowed_roles.to_string()),*],
					});
				});
			}
		}
	}

	let (ensure_constructor, ctor_lets, ctor_args, required_container_typeids) =
		gen_constructor(&input);
	let controller_trait_impl = quote! {
		impl #impl_generics murgamu::MurController for #impl_type #where_clause {
			fn routes(
				self: std::sync::Arc<Self>,
				container: std::sync::Arc<murgamu::MurServiceContainer>,
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

				let _ = &container;
				let mut routes: Vec<MurRouteDefinition> = Vec::new();
				let controller = self.clone();

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
			fn create(injects: &murgamu::MurInjects, _container: &murgamu::MurServiceContainer) -> Self {
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

	quote! {
		#input
		#ensure_constructor
		#controller_trait_impl
		#controller_factory_impl
		#deps_impl
	}
}
