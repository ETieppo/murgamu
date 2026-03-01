use super::{has_self, is_constructor};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, FnArg, Ident, ImplItemFn};

pub fn require_self_params(
	method: &ImplItemFn,
	method_inputs: &Punctuated<FnArg, Comma>,
	method_name: &Ident,
) -> TokenStream {
	if !is_constructor(&method.sig.output) && !has_self(method_inputs) {
		let method_name_str = method_name.to_string();
		quote! {
			compile_error!(concat!(
				"Invalid controller method `",
				#method_name_str,
				"`.\n\n",
				"Route handlers in Murgamü must be instance methods and ",
				"receive `&self` as the first parameter.\n\n",
			));
		}
	} else {
		quote! {}
	}
}
