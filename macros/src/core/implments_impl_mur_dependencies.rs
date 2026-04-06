use proc_macro2::TokenStream;
use quote::quote;
use syn::{ImplGenerics, Type, WhereClause};

pub fn implments_impl_mur_dependencies(
	required_container_typeids: Vec<TokenStream>,
	impl_generics: ImplGenerics<'_>,
	impl_type: &Type,
	where_clause: Option<&WhereClause>,
) -> TokenStream {
	if required_container_typeids.is_empty() {
		quote! {
			impl #impl_generics murgamu::MurDependencies for #impl_type #where_clause {
				fn dependencies() -> &'static [std::any::TypeId] { &[] }
			}
		}
	} else {
		quote! {
			impl #impl_generics murgamu::MurDependencies for #impl_type #where_clause {
				fn dependencies() -> &'static [std::any::TypeId] {
					static DEPS: std::sync::OnceLock<Vec<std::any::TypeId>> = std::sync::OnceLock::new();
					DEPS.get_or_init(|| vec![#(#required_container_typeids),*]).as_slice()
				}
			}
		}
	}
}
