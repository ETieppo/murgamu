use crate::core::{InjectSpec, infer_injects_from_fields, normalize_manual_inject};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ImplGenerics, ItemStruct, Type, TypeGenerics, WhereClause};

pub fn gen_deps_impl(
	input: &ItemStruct,
	struct_name: &Ident,
	impl_generics: &ImplGenerics<'_>,
	ty_generics: &TypeGenerics<'_>,
	where_clause: Option<&WhereClause>,
) -> (
	TokenStream,
	TokenStream,
	Vec<TokenStream>,
	Vec<TokenStream>,
	Vec<Ident>,
	Ident,
	Vec<InjectSpec>,
) {
	let injects_spec: Vec<InjectSpec> = infer_injects_from_fields(input);
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

	let inject_lets: Vec<TokenStream> = injects_spec
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

	let required_container_deps: Vec<TokenStream> = injects_spec
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

	(
		deps_impl,
		create_expr,
		required_container_deps,
		inject_lets,
		inject_vars,
		injects_param,
		injects_spec,
	)
}
