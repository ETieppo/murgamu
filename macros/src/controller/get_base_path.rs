use crate::core::normalize_path;
use syn::Lit;

pub fn get_base_path(args: proc_macro::TokenStream) -> String {
	if !args.is_empty() {
		let lit: Lit = syn::parse(args).unwrap_or(Lit::Str(syn::LitStr::new(
			"/",
			proc_macro2::Span::call_site(),
		)));

		if let Lit::Str(lit_str) = lit {
			normalize_path(&lit_str.value())
		} else {
			String::from("/")
		}
	} else {
		String::from("/")
	}
}
