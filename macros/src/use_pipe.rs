use proc_macro::TokenStream;
use quote::quote;

pub fn use_pipe_impl(_args: TokenStream, _input: TokenStream) -> TokenStream {
	TokenStream::from(quote! {})
}
