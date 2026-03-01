use crate::core::{extract_arc_inner, extract_option_inner};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ItemImpl;

pub fn gen_constructor(
	input: &ItemImpl,
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

	let mut ensure_new = if new_fn.is_some() {
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
				ensure_new = quote! {
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

			if let Some(inner) = extract_option_inner(ty) {
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

	(ensure_new, ctor_lets, ctor_args, required_container_typeids)
}
