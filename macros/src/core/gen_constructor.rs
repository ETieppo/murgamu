use crate::core::{extract_arc_inner, extract_option_inner};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemImpl, Pat};

pub fn gen_constructor(
	input: &ItemImpl,
) -> (
	TokenStream,
	Vec<TokenStream>,
	Vec<TokenStream>,
	Vec<TokenStream>,
) {
	gen_constructor_with_fallback(input, Some("Controller must define `fn new(...) -> Self`."))
}

pub fn gen_constructor_with_fallback(
	input: &ItemImpl,
	require_new_msg: Option<&str>,
) -> (
	TokenStream,
	Vec<TokenStream>,
	Vec<TokenStream>,
	Vec<TokenStream>,
) {
	let new_fn = input.items.iter().find_map(|item| {
		if let syn::ImplItem::Fn(f) = item
			&& f.sig.ident == "new"
		{
			return Some(f);
		}

		None
	});

	let mut ensure_constructor = if new_fn.is_some() {
		quote! {}
	} else {
		match require_new_msg {
			Some(msg) => quote! { compile_error!(#msg); },
			None => quote! {},
		}
	};
	let mut ctor_lets: Vec<TokenStream> = Vec::new();
	let mut ctor_args: Vec<TokenStream> = Vec::new();
	let mut required_container_typeids: Vec<TokenStream> = Vec::new();

	if let Some(new_fn) = new_fn {
		for arg in &new_fn.sig.inputs {
			if matches!(arg, syn::FnArg::Receiver(_)) {
				ensure_constructor = quote! {
					compile_error!("Controller constructor `new(...)` must be an associated function (no self parameter).");
				};
			}
		}

		for (idx, fnarg) in new_fn.sig.inputs.iter().enumerate() {
			let syn::FnArg::Typed(pat_type) = fnarg else {
				continue;
			};

			if let Pat::Ident(pat_ident) = pat_type.pat.as_ref()
				&& pat_ident.ident.to_string().starts_with('_')
			{
				let var = format_ident!("__mur_ctor_{idx}");
				let ty = &pat_type.ty;
				ctor_lets.push(quote! { let #var = <#ty as Default>::default(); });
				ctor_args.push(quote! { #var });
				continue;
			}

			let var = format_ident!("__mur_ctor_{idx}");
			let ty = &pat_type.ty;

			if let Some(inner) = extract_option_inner(ty) {
				if let Some(arc_inner) = extract_arc_inner(inner) {
					ctor_lets.push(quote! { let #var = _container.get::<#arc_inner>(); });
					ctor_args.push(quote! { #var });
				} else {
					ctor_lets.push(
						quote! { let #var = _container.get::<#inner>().map(|arc| (*arc).clone()); },
					);
					ctor_args.push(quote! { #var });
				}
				continue;
			}

			if let Some(inner) = extract_arc_inner(ty) {
				ctor_lets.push(quote! { let #var = _container.get_required::<#inner>(); });
				ctor_args.push(quote! { #var });
				required_container_typeids.push(quote!(std::any::TypeId::of::<#inner>()));
				continue;
			}

			ctor_lets.push(quote! { let #var = (*_container.get_required::<#ty>()).clone(); });
			ctor_args.push(quote! { #var });
			required_container_typeids.push(quote!(std::any::TypeId::of::<#ty>()));
		}
	}

	(
		ensure_constructor,
		ctor_lets,
		ctor_args,
		required_container_typeids,
	)
}
