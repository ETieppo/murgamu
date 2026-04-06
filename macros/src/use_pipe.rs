use proc_macro::TokenStream;
use quote::quote;

pub fn use_pipe_impl(args: TokenStream, input: TokenStream) -> TokenStream {
	TokenStream::from(quote! {})
}
