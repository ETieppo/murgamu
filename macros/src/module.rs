use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
	parse::{Parse, ParseStream},
	parse_macro_input,
	punctuated::Punctuated,
	Ident, ItemStruct, Token,
};

#[derive(Debug)]
struct ModuleArgs {
	controllers: Vec<Ident>,
	services: Vec<Ident>,
}

impl Parse for ModuleArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut controllers = Vec::new();
		let mut services = Vec::new();

		while !input.is_empty() {
			let key: Ident = input.parse()?;
			input.parse::<Token![=]>()?;

			let content;
			syn::bracketed!(content in input);
			let items: Punctuated<Ident, Token![,]> =
				content.parse_terminated(Ident::parse, Token![,])?;

			match key.to_string().as_str() {
				"controllers" => controllers.extend(items),
				"services" => services.extend(items),
				_ => {
					return Err(syn::Error::new_spanned(
						key,
						"unknown module attribute key. Allowed keys: services, controllers",
					));
				}
			}

			if input.peek(Token![,]) {
				input.parse::<Token![,]>()?;
			}
		}

		Ok(ModuleArgs {
			controllers,
			services,
		})
	}
}

pub fn module_impl(args: TokenStream, input: TokenStream) -> TokenStream {
	let args = parse_macro_input!(args as ModuleArgs);
	let input = parse_macro_input!(input as ItemStruct);
	let output = generate_module_impl(&input, &args);
	TokenStream::from(output)
}

fn generate_module_impl(input: &ItemStruct, args: &ModuleArgs) -> TokenStream2 {
	let module_name = &input.ident;
	let vis = &input.vis;
	let module_name_str = module_name.to_string();

	let service_fields: Vec<(Ident, &Ident, Ident)> = args
		.services
		.iter()
		.map(|s| {
			let field_name = to_snake_case_ident(s);
			let helper_name = format_ident!("__mur_service_{}", field_name);
			(field_name, s, helper_name)
		})
		.collect();

	let struct_fields: Vec<TokenStream2> = service_fields
		.iter()
		.map(|(field_name, service_type, _)| {
			quote! { #field_name: std::sync::OnceLock<std::sync::Arc<#service_type>> }
		})
		.collect();

	let service_inits: Vec<TokenStream2> = service_fields
		.iter()
		.map(|(field_name, _, _)| quote! { #field_name: std::sync::OnceLock::new() })
		.collect();

	let service_helpers: Vec<TokenStream2> = service_fields
		.iter()
		.map(|(field_name, service_type, helper_name)| {
			quote! {
				fn #helper_name(&self, _injects: &MurInjects) -> std::sync::Arc<#service_type> {
					self.#field_name
						.get_or_init(|| {
							std::sync::Arc::new(<#service_type as MurServiceFactory>::create(_injects))
						})
						.clone()
				}
			}
		})
		.collect();

	let controller_insts: Vec<TokenStream2> = args
		.controllers
		.iter()
		.map(|controller_type| {
			let service_args: Vec<TokenStream2> = service_fields
				.iter()
				.map(|(_, _, helper_name)| quote! { self.#helper_name(_injects) })
				.collect();

			quote! {
				std::sync::Arc::new(#controller_type::new(#(#service_args),*))
			}
		})
		.collect();

	let service_regs: Vec<TokenStream2> = service_fields
		.iter()
		.map(|(_, service_type, helper_name)| {
			quote! {
				(
					std::any::TypeId::of::<#service_type>(),
					self.#helper_name(_injects) as std::sync::Arc<dyn MurService>
				)
			}
		})
		.collect();

	let service_getters: Vec<TokenStream2> = service_fields
		.iter()
		.map(|(_, service_type, helper_name)| {
			quote! {
				if std::any::TypeId::of::<T>() == std::any::TypeId::of::<#service_type>() {
					let service_arc = self.#helper_name(&empty_injects);

					let ptr = std::sync::Arc::into_raw(service_arc) as *const T;
					return Some(unsafe { std::sync::Arc::from_raw(ptr) });
				}
			}
		})
		.collect();

	let service_method_body = if service_getters.is_empty() {
		quote! { None }
	} else {
		quote! {
			let empty_injects = MurInjects::default();
			#(#service_getters)*
			None
		}
	};
	quote! {
		#vis struct #module_name {
			#(#struct_fields),*
		}

		impl #module_name {
			pub fn new() -> Self {
				Self {
					#(#service_inits),*
				}
			}

			#(#service_helpers)*

			pub fn service<T: MurService + 'static>(&self) -> Option<std::sync::Arc<T>> {
				#service_method_body
			}
		}

		impl Default for #module_name {
			fn default() -> Self {
				Self::new()
			}
		}

		impl MurModule for #module_name {
			fn name(&self) -> &str {
				#module_name_str
			}

			fn controllers_with_injects(
				&self,
				_injects: &MurInjects,
			) -> Vec<std::sync::Arc<dyn MurController>> {
				vec![#(#controller_insts),*]
			}

			fn controllers(&self) -> Vec<std::sync::Arc<dyn MurController>> {
				self.controllers_with_injects(&MurInjects::default())
			}

			fn services_with_injects(
				&self,
				_injects: &MurInjects,
			) -> Vec<(std::any::TypeId, std::sync::Arc<dyn MurService>)> {
				vec![#(#service_regs),*]
			}

			fn services(&self) -> Vec<(std::any::TypeId, std::sync::Arc<dyn MurService>)> {
				self.services_with_injects(&MurInjects::default())
			}
		}
	}
}

fn to_snake_case_ident(ident: &Ident) -> Ident {
	let name = ident.to_string();
	let snake = to_snake_case(&name);
	format_ident!("{}", snake)
}

fn to_snake_case(s: &str) -> String {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_to_snake_case() {
		assert_eq!(to_snake_case("UserService"), "user_service");
		assert_eq!(to_snake_case("HTTPServer"), "h_t_t_p_server");
		assert_eq!(to_snake_case("MyAPI"), "my_a_p_i");
		assert_eq!(to_snake_case("Simple"), "simple");
	}
}
