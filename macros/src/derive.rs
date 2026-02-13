use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

pub fn derive_dto_impl(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let output = generate_dto_impl(&input);
	TokenStream::from(output)
}

fn generate_dto_impl(input: &DeriveInput) -> TokenStream2 {
	let name = &input.ident;
	let generics = &input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	quote! {
		impl #impl_generics #name #ty_generics #where_clause {
			pub fn validate(&self) -> Result<(), String> {
				Ok(())
			}
		}
	}
}

pub fn derive_entity_impl(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let output = generate_entity_impl(&input);
	TokenStream::from(output)
}

fn generate_entity_impl(input: &DeriveInput) -> TokenStream2 {
	let name = &input.ident;
	let generics = &input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let has_id_field = match &input.data {
		syn::Data::Struct(data) => data.fields.iter().any(|f| {
			f.ident
				.as_ref()
				.map(|i| i == "id")
				.unwrap_or(false)
		}),
		_ => false,
	};

	if has_id_field {
		quote! {
			impl #impl_generics #name #ty_generics #where_clause {

				pub fn id(&self) -> &str {
					&self.id
				}
			}
		}
	} else {
		quote! {
			impl #impl_generics #name #ty_generics #where_clause {
			}
		}
	}
}
