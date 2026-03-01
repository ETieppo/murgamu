use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn interceptor_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ItemStruct);
	let output = generate_interceptor_impl(&input);
	TokenStream::from(output)
}

fn generate_interceptor_impl(input: &ItemStruct) -> TokenStream2 {
	let struct_name = &input.ident;
	let generics = &input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	quote! {
		#input

		impl #impl_generics MurInterceptor for #struct_name #ty_generics #where_clause {
			fn name(&self) -> &str {
				stringify!(#struct_name)
			}
		}
	}
}

#[allow(dead_code)]
pub fn timing_interceptor_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ItemStruct);
	let output = generate_timing_interceptor_impl(&input);
	TokenStream::from(output)
}

#[allow(dead_code)]
fn generate_timing_interceptor_impl(input: &ItemStruct) -> TokenStream2 {
	let struct_name = &input.ident;
	let generics = &input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	quote! {
		#input

		impl #impl_generics MurInterceptor for #struct_name #ty_generics #where_clause {
			fn before(
				&self,
				ctx: &MurRequestContext,
			) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), MurError>> + Send + '_>> {
				let method = ctx.method().clone();
				let uri = ctx.uri().clone();
				Box::pin(async move {
					println!("[TIMING] {} {} - started", method, uri);
					Ok(())
				})
			}

			fn name(&self) -> &str {
				stringify!(#struct_name)
			}
		}
	}
}
