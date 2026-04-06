use crate::injectable::injectable_impl;
use proc_macro::TokenStream;

/// Alias for `#[injectable]`. Prefer using `#[injectable]` directly.
pub fn service_impl(args: TokenStream, input: TokenStream) -> TokenStream {
	injectable_impl(args, input)
}
