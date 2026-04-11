use crate::core::implments_struct_mur_dependencies;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

pub fn injectable_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ItemStruct);
	TokenStream::from(injectable_impls(&input))
}

pub fn injectable_impls(input: &ItemStruct) -> TokenStream2 {
	let struct_name = &input.ident;
	let generics = &input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
	let (deps_impl, create_expr, inject_lets, injects_param) = implments_struct_mur_dependencies(
		input,
		struct_name,
		&impl_generics,
		&ty_generics,
		where_clause,
	);

	// Emit `#[derive(Clone)]` automatically unless the user already wrote it.
	// Services are shared via Arc and cloning must be cheap; this removes boilerplate
	// that would otherwise be mandatory for plain-T controller injection.
	let auto_clone = if has_clone_derive(input) {
		quote! {}
	} else {
		quote! { #[derive(Clone)] }
	};

	quote! {
		#auto_clone
		#input

		impl #impl_generics murgamu::MurService for #struct_name #ty_generics #where_clause {
			fn as_any(&self) -> &dyn std::any::Any {
				self
			}
		}

		impl #impl_generics murgamu::MurInjectable for #struct_name #ty_generics #where_clause {
			fn as_any(&self) -> &dyn std::any::Any {
				self
			}
		}

		impl #impl_generics murgamu::MurServiceFactory for #struct_name #ty_generics #where_clause {
			fn __create_factory(
				#injects_param: &murgamu::MurInjects,
				_container: &murgamu::MurServiceContainer,
			) -> Self {
				#(#inject_lets)*
				#create_expr
			}
		}

		#deps_impl
	}
}

/// Returns `true` if the struct already has `Clone` in any `#[derive(...)]` attribute.
pub(crate) fn has_clone_derive(input: &ItemStruct) -> bool {
	input.attrs.iter().any(|attr| {
		if !attr.path().is_ident("derive") {
			return false;
		}
		let Ok(list) = attr.meta.require_list() else {
			return false;
		};
		list.tokens
			.to_string()
			.split(',')
			.any(|s| s.trim() == "Clone")
	})
}
