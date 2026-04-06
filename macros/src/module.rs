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
	imports: Vec<Ident>,
	controllers: Vec<Ident>,
	services: Vec<Ident>,
	exports: Vec<Ident>,
}

impl Parse for ModuleArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut imports = Vec::new();
		let mut controllers = Vec::new();
		let mut services = Vec::new();
		let mut exports = Vec::new();

		while !input.is_empty() {
			let key: Ident = input.parse()?;
			input.parse::<Token![=]>()?;

			let content;
			syn::bracketed!(content in input);
			let items: Punctuated<Ident, Token![,]> =
				content.parse_terminated(Ident::parse, Token![,])?;

			match key.to_string().as_str() {
				"imports" => imports.extend(items),
				"controllers" => controllers.extend(items),
				"services" => services.extend(items),
				"exports" => exports.extend(items),
				_ => {
					return Err(syn::Error::new_spanned(
                        key,
                        "unknown module attribute key. Allowed keys: imports, controllers, services, exports",
                    ));
				}
			}

			if input.peek(Token![,]) {
				input.parse::<Token![,]>()?;
			}
		}

		Ok(ModuleArgs {
			imports,
			controllers,
			services,
			exports,
		})
	}
}

pub fn module_impl(args: TokenStream, input: TokenStream) -> TokenStream {
	let args = parse_macro_input!(args as ModuleArgs);
	let input = parse_macro_input!(input as ItemStruct);
	TokenStream::from(generate_module_impl(&input, &args))
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
				fn #helper_name(
					&self,
					_injects: &murgamu::MurInjects,
					_container: &murgamu::MurServiceContainer,
				) -> std::sync::Arc<#service_type> {
					self.#field_name
						.get_or_init(|| {
							std::sync::Arc::new(
								<#service_type as murgamu::MurServiceFactory>::create(_injects, _container)
							)
						})
						.clone()
				}
			}
		})
		.collect();

	let import_insts: Vec<TokenStream2> = args
		.imports
		.iter()
		.map(|m| {
			quote! { std::sync::Arc::new(#m::new()) as std::sync::Arc<dyn murgamu::MurModule> }
		})
		.collect();

	let export_typeids: Vec<TokenStream2> = args
		.exports
		.iter()
		.map(|s| quote! { std::any::TypeId::of::<#s>() })
		.collect();

	let controller_insts: Vec<TokenStream2> = args
		.controllers
		.iter()
		.map(|c| {
			quote! {
				std::sync::Arc::new(
					<#c as murgamu::MurControllerFactory>::create(_injects, _container)
				) as std::sync::Arc<dyn murgamu::MurController>
			}
		})
		.collect();

	let module_service_typeids: Vec<TokenStream2> = args
		.services
		.iter()
		.map(|s| quote!(std::any::TypeId::of::<#s>()))
		.collect();

	let pending_inits: Vec<TokenStream2> = service_fields
        .iter()
        .map(|(_, service_type, helper_name)| {
            quote! {
                (
                    std::any::TypeId::of::<#service_type>(),
                    <#service_type as murgamu::MurDependencies>::dependencies(),
                    (|m: &#module_name, inj: &murgamu::MurInjects, c: &murgamu::MurServiceContainer|
                        -> std::sync::Arc<dyn murgamu::MurService> {
                        m.#helper_name(inj, c) as std::sync::Arc<dyn murgamu::MurService>
                    }) as fn(&#module_name, &murgamu::MurInjects, &murgamu::MurServiceContainer) -> std::sync::Arc<dyn murgamu::MurService>
                )
            }
        })
        .collect();

	quote! {
		#vis struct #module_name {
			#(#struct_fields),*
		}

		impl #module_name {
			pub fn new() -> Self {
				Self { #(#service_inits),* }
			}

			#(#service_helpers)*
		}

		impl Default for #module_name {
			fn default() -> Self { Self::new() }
		}

		impl murgamu::MurModule for #module_name {
			fn name(&self) -> &str { #module_name_str }

			fn imports(&self) -> Vec<std::sync::Arc<dyn murgamu::MurModule>> {
				vec![#(#import_insts),*]
			}

			fn exports(&self) -> Vec<std::any::TypeId> {
				vec![#(#export_typeids),*]
			}

			fn controllers_with_injects(
				&self,
				_injects: &murgamu::MurInjects,
				_container: &murgamu::MurServiceContainer,
			) -> Vec<std::sync::Arc<dyn murgamu::MurController>> {
				vec![#(#controller_insts),*]
			}

			fn controllers(&self) -> Vec<std::sync::Arc<dyn murgamu::MurController>> {
				let injects = murgamu::MurInjects::default();
				let container = murgamu::MurServiceContainer::new();
				self.controllers_with_injects(&injects, &container)
			}

			fn services_with_injects(
				&self,
				_injects: &murgamu::MurInjects,
				_container: &murgamu::MurServiceContainer,
			) -> Vec<(std::any::TypeId, std::sync::Arc<dyn murgamu::MurService>)> {
				let mut out: Vec<(std::any::TypeId, std::sync::Arc<dyn murgamu::MurService>)> = Vec::new();
				let mut local = _container.clone();

				let module_ids: std::collections::HashSet<std::any::TypeId> =
					vec![#(#module_service_typeids),*].into_iter().collect();

				let mut created: std::collections::HashSet<std::any::TypeId> =
					std::collections::HashSet::new();

				let mut pending: Vec<(
					std::any::TypeId,
					&'static [std::any::TypeId],
					fn(&#module_name, &murgamu::MurInjects, &murgamu::MurServiceContainer) -> std::sync::Arc<dyn murgamu::MurService>
				)> = vec![#(#pending_inits),*];

				while !pending.is_empty() {
					let mut progressed = false;
					let mut i = 0usize;

					while i < pending.len() {
						let (tid, deps, maker) = pending[i];
						let ok = deps.iter().all(|d| !module_ids.contains(d) || created.contains(d));

						if ok {
							let svc = maker(self, _injects, &local);
							local.register_dyn_with_id(tid, svc.clone());
							out.push((tid, svc));
							created.insert(tid);
							pending.swap_remove(i);
							progressed = true;
						} else {
							i += 1;
						}
					}

					if !progressed {
						panic!(
							"Could not resolve module service dependencies (cycle or missing internal provider). Pending TypeIds: {:?}",
							pending.iter().map(|(t, _, _)| *t).collect::<Vec<_>>()
						);
					}
				}

				out
			}

			fn services(&self) -> Vec<(std::any::TypeId, std::sync::Arc<dyn murgamu::MurService>)> {
				vec![]
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
