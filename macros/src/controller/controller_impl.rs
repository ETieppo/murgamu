use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemImpl, Lit, Meta, MetaList};

use crate::types::ParamInfo;
use crate::utils::{analyze_parameter, has_self, is_constructor, normalize_path};

fn extract_arc_inner(ty: &syn::Type) -> Option<syn::Type> {
	use syn::{GenericArgument, PathArguments, Type};

	let Type::Path(tp) = ty else { return None };
	let seg = tp.path.segments.last()?;
	if seg.ident != "Arc" {
		return None;
	}

	let PathArguments::AngleBracketed(ab) = &seg.arguments else {
		return None;
	};

	if ab.args.len() != 1 {
		return None;
	}

	match ab.args.first()? {
		GenericArgument::Type(inner) => Some(inner.clone()),
		_ => None,
	}
}

fn extract_option_inner(ty: &syn::Type) -> Option<syn::Type> {
	use syn::{GenericArgument, PathArguments, Type};

	let Type::Path(tp) = ty else { return None };
	let seg = tp.path.segments.last()?;
	if seg.ident != "Option" {
		return None;
	}

	let PathArguments::AngleBracketed(ab) = &seg.arguments else {
		return None;
	};

	if ab.args.len() != 1 {
		return None;
	}

	match ab.args.first()? {
		GenericArgument::Type(inner) => Some(inner.clone()),
		_ => None,
	}
}

fn option_arc_inner(ty: &syn::Type) -> Option<syn::Type> {
	let opt_inner = extract_option_inner(ty)?;
	extract_arc_inner(&opt_inner)
}

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
	let generics = &input.generics;
	let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

	let mut route_registrations: Vec<TokenStream> = Vec::new();

	for item in &input.items {
		let syn::ImplItem::Fn(method) = item else {
			continue;
		};

		let method_name = &method.sig.ident;
		let method_inputs = &method.sig.inputs;

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

		let mut is_public = false;
		let mut allowed_roles: Vec<String> = Vec::new();

		for attr in &method.attrs {
			if attr.path().is_ident("public") {
				is_public = true;
			}

			if attr.path().is_ident("role") {
				if let Meta::List(meta_list) = &attr.meta {
					let tokens_str = meta_list.tokens.to_string();
					for part in tokens_str.split(',') {
						let role = part.trim();
						if let Some(name) = role.split("::").last() {
							allowed_roles.push(name.to_string());
						}
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
					"get" | "post" | "put" | "delete" | "patch" | "head" | "options"
				) {
					continue;
				}

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
				for input_arg in method_inputs.iter() {
					if let FnArg::Typed(pat_type) = input_arg {
						params.push(analyze_parameter(pat_type));
					}
				}

				let handler_code = super::generate_handler_code(method_name, &params);

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

	let new_fn = input.items.iter().find_map(|item| {
		if let syn::ImplItem::Fn(f) = item {
			if f.sig.ident == "new" {
				return Some(f);
			}
		}
		None
	});

	let ensure_new = if new_fn.is_some() {
		quote! {}
	} else {
		quote! { compile_error!("Controller must define `fn new(...) -> Self`."); }
	};

	let mut ctor_lets: Vec<TokenStream> = Vec::new();
	let mut ctor_args: Vec<TokenStream> = Vec::new();
	let mut required_container_typeids: Vec<TokenStream> = Vec::new();

	if let Some(new_fn) = new_fn {
		for arg in &new_fn.sig.inputs {
			if matches!(arg, syn::FnArg::Receiver(_)) {
				return quote! {
					compile_error!("Controller constructor `new(...)` must be an associated function (no self parameter).");
				};
			}
		}

		for (idx, fnarg) in new_fn.sig.inputs.iter().enumerate() {
			let syn::FnArg::Typed(pat_type) = fnarg else {
				continue;
			};

			let var = format_ident!("__mur_ctor_{idx}");
			let ty = &pat_type.ty;

			if let Some(inner) = option_arc_inner(ty) {
				ctor_lets.push(quote! { let #var = _container.get::<#inner>(); });
				ctor_args.push(quote! { #var });
				continue;
			}

			if let Some(inner) = extract_arc_inner(ty) {
				ctor_lets.push(quote! { let #var = _container.get_required::<#inner>(); });
				ctor_args.push(quote! { #var });
				required_container_typeids.push(quote!(std::any::TypeId::of::<#inner>()));
				continue;
			}

			ctor_lets.push(quote! { let #var = injects.get_required::<#ty>(); });
			ctor_args.push(quote! { #var });
		}
	}

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

	let deps_impl = if required_container_typeids.is_empty() {
		quote! {
			impl #impl_generics murgamu::MurDependencies for #impl_type #where_clause {
				fn dependencies() -> &'static [std::any::TypeId] { &[] }
			}
		}
	} else {
		quote! {
			impl #impl_generics murgamu::MurDependencies for #impl_type #where_clause {
				fn dependencies() -> &'static [std::any::TypeId] {
					static DEPS: std::sync::OnceLock<Vec<std::any::TypeId>> = std::sync::OnceLock::new();
					DEPS.get_or_init(|| vec![#(#required_container_typeids),*]).as_slice()
				}
			}
		}
	};

	quote! {
		#input
		#ensure_new
		#controller_trait_impl
		#controller_factory_impl
		#deps_impl
	}
}
