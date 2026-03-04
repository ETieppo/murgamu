use quote::format_ident;
use syn::Ident;

pub fn to_snake_case_ident(ident: &Ident) -> Ident {
	let name = ident.to_string();
	let snake = to_snake_case(&name);
	format_ident!("{}", snake)
}

pub fn to_snake_case(s: &str) -> String {
	let mut result = String::new();
	for (i, c) in s.chars().enumerate() {
		if c.is_uppercase() {
			if i > 0 {
				result.push('_');
			}
			result.push(c.to_lowercase().next().unwrap());
		} else {
			result.push(c);
		}
	}
	result
}
