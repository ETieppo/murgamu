use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{GenericArgument, ItemStruct, PathArguments, Type};

#[derive(Clone)]
struct InjectSpec {
	ty: Type,
	optional: bool,
	via_container: bool,
}

fn extract_arc_inner(ty: &Type) -> Option<&Type> {
	let path = match ty {
		Type::Path(type_path) => &type_path.path,
		_ => return None,
	};
	let seg = path.segments.last()?;
	if seg.ident != "Arc" {
		return None;
	}
	let args = match &seg.arguments {
		PathArguments::AngleBracketed(args) => args,
		_ => return None,
	};
	if args.args.len() != 1 {
		return None;
	}
	match args.args.first()? {
		GenericArgument::Type(inner) => Some(inner),
		_ => None,
	}
}

fn extract_option_inner(ty: &Type) -> Option<&Type> {
	let path = match ty {
		Type::Path(type_path) => &type_path.path,
		_ => return None,
	};
	let seg = path.segments.last()?;
	if seg.ident != "Option" {
		return None;
	}
	let args = match &seg.arguments {
		PathArguments::AngleBracketed(args) => args,
		_ => return None,
	};
	if args.args.len() != 1 {
		return None;
	}
	match args.args.first()? {
		GenericArgument::Type(inner) => Some(inner),
		_ => None,
	}
}

fn normalize_manual_inject(ty: &Type) -> InjectSpec {
	if let Some(opt_inner) = extract_option_inner(ty) {
		if let Some(arc_inner) = extract_arc_inner(opt_inner) {
			return InjectSpec {
				ty: arc_inner.clone(),
				optional: true,
				via_container: true,
			};
		}
	}

	if let Some(arc_inner) = extract_arc_inner(ty) {
		return InjectSpec {
			ty: arc_inner.clone(),
			optional: false,
			via_container: true,
		};
	}

	InjectSpec {
		ty: ty.clone(),
		optional: false,
		via_container: false,
	}
}

fn infer_injects_from_fields(input: &ItemStruct) -> Vec<InjectSpec> {
	input
		.fields
		.iter()
		.filter_map(|field| {
			if let Some(opt_inner) = extract_option_inner(&field.ty) {
				if let Some(arc_inner) = extract_arc_inner(opt_inner) {
					return Some(InjectSpec {
						ty: arc_inner.clone(),
						optional: true,
						via_container: true,
					});
				}
				return None;
			}

			if let Some(arc_inner) = extract_arc_inner(&field.ty) {
				return Some(InjectSpec {
					ty: arc_inner.clone(),
					optional: false,
					via_container: true,
				});
			}

			None
		})
		.collect()
}

pub fn generate_service_impl(
	input: &ItemStruct,
	injects: &[Type],
	has_injects: bool,
) -> TokenStream2 {
	let struct_name = &input.ident;
	let generics = &input.generics;
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let injects_spec: Vec<InjectSpec> = if has_injects {
		injects.iter().map(normalize_manual_inject).collect()
	} else {
		infer_injects_from_fields(input)
	};

	let injects_param = if injects_spec.is_empty() {
		format_ident!("_injects")
	} else {
		format_ident!("injects")
	};

	let inject_vars: Vec<_> = injects_spec
		.iter()
		.enumerate()
		.map(|(idx, _)| format_ident!("__mur_inject_{idx}"))
		.collect();

	let inject_lets: Vec<TokenStream2> = injects_spec
		.iter()
		.zip(inject_vars.iter())
		.map(|(spec, var)| {
			let ty = &spec.ty;

			if spec.via_container {
				if spec.optional {
					quote! { let #var = _container.get::<#ty>(); }
				} else {
					quote! { let #var = _container.get_required::<#ty>(); }
				}
			} else if spec.optional {
				quote! { let #var = #injects_param.get::<#ty>(); }
			} else {
				quote! { let #var = #injects_param.get_required::<#ty>(); }
			}
		})
		.collect();

	let create_expr = if injects_spec.is_empty() {
		quote! { Self::new() }
	} else {
		quote! { Self::new(#(#inject_vars),*) }
	};
	let required_container_deps: Vec<TokenStream2> = injects_spec
		.iter()
		.filter(|spec| spec.via_container && !spec.optional)
		.map(|spec| {
			let ty = &spec.ty;
			quote!(std::any::TypeId::of::<#ty>())
		})
		.collect();

	let deps_impl = if required_container_deps.is_empty() {
		quote! {
			impl #impl_generics murgamu::MurDependencies for #struct_name #ty_generics #where_clause {
				fn dependencies() -> &'static [std::any::TypeId] {
					&[]
				}
			}
		}
	} else {
		quote! {
			impl #impl_generics murgamu::MurDependencies for #struct_name #ty_generics #where_clause {
				fn dependencies() -> &'static [std::any::TypeId] {
					static DEPS: std::sync::OnceLock<Vec<std::any::TypeId>> = std::sync::OnceLock::new();
					DEPS.get_or_init(|| vec![#(#required_container_deps),*]).as_slice()
				}
			}
		}
	};

	quote! {
		#input

		impl #impl_generics murgamu::MurService for #struct_name #ty_generics #where_clause {
			fn as_any(&self) -> &dyn std::any::Any {
				self
			}
		}

		impl #impl_generics murgamu::MurServiceFactory for #struct_name #ty_generics #where_clause {
			fn create(#injects_param: &murgamu::MurInjects, _container: &murgamu::MurServiceContainer) -> Self {
				#(#inject_lets)*
				#create_expr
			}
		}

		#deps_impl
	}
}
