use crate::core::implments_struct_mur_dependencies;
use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;
use syn::parse_macro_input;

pub fn service_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ItemStruct);
	let struct_name = &input.ident;
	let generics = &input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
	let (deps_impl, create_expr, inject_lets, injects_param) =
		implments_struct_mur_dependencies(
			&input,
			struct_name,
			&impl_generics,
			&ty_generics,
			where_clause,
		);

	TokenStream::from(quote! {
		#input

		impl #impl_generics murgamu::MurService for #struct_name #ty_generics #where_clause {
			fn as_any(&self) -> &dyn std::any::Any {
				self
			}
		}

		impl #impl_generics murgamu::MurServiceFactory for #struct_name #ty_generics #where_clause {
			fn create(#injects_param: &murgamu::MurInjects, _container: &murgamu::MurServiceContainer) -> Self {
				#(#inject_lets)*
				#create_expr
			}
		}

		#deps_impl
	})
}
