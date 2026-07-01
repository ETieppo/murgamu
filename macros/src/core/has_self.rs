use syn::{FnArg, punctuated::Punctuated, token::Comma};

pub fn has_self(method_inputs: &Punctuated<FnArg, Comma>) -> bool {
	method_inputs
		.first()
		.is_some_and(|input| matches!(input, FnArg::Receiver(_)))
}
