use proc_macro::TokenStream;
use syn::{
	parse::{Parse, ParseStream},
	parse_macro_input,
	punctuated::Punctuated,
	Ident, ItemStruct, Token, Type,
};

struct ServiceArgs {
	injects: Vec<Type>,
	has_injects: bool,
}

impl Parse for ServiceArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut injects = Vec::new();
		let mut has_injects = false;

		if input.is_empty() {
			return Ok(Self {
				injects,
				has_injects,
			});
		}

		while !input.is_empty() {
			let key: Ident = input.parse()?;
			input.parse::<Token![=]>()?;

			let content;
			syn::bracketed!(content in input);
			let items: Punctuated<Type, Token![,]> =
				content.parse_terminated(Type::parse, Token![,])?;

			match key.to_string().as_str() {
				"injects" => {
					has_injects = true;
					injects.extend(items);
				}
				_ => {
					return Err(syn::Error::new_spanned(
						key,
						"unknown service attribute key. Allowed keys: injects",
					));
				}
			}

			if input.peek(Token![,]) {
				input.parse::<Token![,]>()?;
			}
		}

		Ok(Self {
			injects,
			has_injects,
		})
	}
}

pub fn service_impl(args: TokenStream, input: TokenStream) -> TokenStream {
	let args = parse_macro_input!(args as ServiceArgs);
	let input = parse_macro_input!(input as ItemStruct);
	let output = super::generate_service_impl(&input, &args.injects, args.has_injects);
	TokenStream::from(output)
}
