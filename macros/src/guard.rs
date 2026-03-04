use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

use crate::core::implments_struct_mur_dependencies;

pub fn guard_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ItemStruct);
	let output = generate_guard_impl(&input);
	TokenStream::from(output)
}

fn generate_guard_impl(input: &ItemStruct) -> TokenStream2 {
	let struct_name = &input.ident;
	let generics = &input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
	let (deps_impl, create_expr, inject_lets, injects_param) =
		implments_struct_mur_dependencies(
			input,
			struct_name,
			&impl_generics,
			&ty_generics,
			where_clause,
		);

	quote! {
		#input
		#deps_impl

		impl #impl_generics MurGuard for #struct_name #ty_generics #where_clause {
			fn as_any(&self) -> &dyn std::any::Any {
				self
			}

			// fn new() -> {
			// 	return Self {}
			// }

			fn check_can_activate<'a>(
				&'a self,
				ctx: &'a MurRequestContext,
			) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + 'a>> {
				Box::pin(async move {
					self.can_activate(ctx).await
				})
			}
		}

		impl #impl_generics murgamu::MurGuardFactory for #struct_name #ty_generics #where_clause {
			fn create(#injects_param: &murgamu::MurInjects, _container: &murgamu::MurServiceContainer) -> Self {
				#(#inject_lets)*
				#create_expr
			}
		}
	}
}
