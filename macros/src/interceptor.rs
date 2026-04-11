use crate::core::implments_struct_mur_dependencies;
use crate::injectable::has_clone_derive;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

pub fn interceptor_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ItemStruct);
	let struct_name = &input.ident;
	let generics = &input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
	let (deps_impl, create_expr, inject_lets, injects_param) = implments_struct_mur_dependencies(
		&input,
		struct_name,
		&impl_generics,
		&ty_generics,
		where_clause,
	);

	let auto_clone = if has_clone_derive(&input) {
		quote! {}
	} else {
		quote! { #[derive(Clone)] }
	};

	TokenStream::from(quote! {
		#auto_clone
		#input

		impl #impl_generics murgamu::MurInterceptor for #struct_name #ty_generics #where_clause {
			fn name(&self) -> &str {
				stringify!(#struct_name)
			}
		}

		impl #impl_generics murgamu::MurInterceptorFactory for #struct_name #ty_generics #where_clause {
			fn __create_factory(
				#injects_param: &murgamu::MurInjects,
				_container: &murgamu::MurServiceContainer,
			) -> Self {
				#(#inject_lets)*
				#create_expr
			}
		}

		#deps_impl
	})
}
