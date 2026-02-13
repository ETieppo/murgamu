use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn injectable_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ItemStruct);
	let output = generate_injectable_impl(&input);
	TokenStream::from(output)
}

fn generate_injectable_impl(input: &ItemStruct) -> TokenStream2 {
	let struct_name = &input.ident;
	let generics = &input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	quote! {
		#input

		impl #impl_generics MurService for #struct_name #ty_generics #where_clause {
			fn as_any(&self) -> &dyn std::any::Any {
				self
			}
		}

		impl #impl_generics MurInjectable for #struct_name #ty_generics #where_clause {
			fn as_any(&self) -> &dyn std::any::Any {
				self
			}
		}
	}
}
