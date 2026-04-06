use crate::core::{
	InjectSpec, extract_arc_inner, extract_option_inner, infer_inject_from_field::is_skipped_field,
	infer_injects_from_fields,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, Ident, ImplGenerics, ItemStruct, TypeGenerics, WhereClause};

pub fn implments_struct_mur_dependencies(
	input: &ItemStruct,
	struct_name: &Ident,
	impl_generics: &ImplGenerics<'_>,
	ty_generics: &TypeGenerics<'_>,
	where_clause: Option<&WhereClause>,
) -> (TokenStream, TokenStream, Vec<TokenStream>, Ident) {
	let is_tuple_struct = matches!(&input.fields, Fields::Unnamed(_));

	let injects_spec: Vec<InjectSpec> = infer_injects_from_fields(input);
	let has_skipped_fields =
		!is_tuple_struct && input.fields.iter().any(is_skipped_field);
	let any_from_injects = injects_spec.iter().any(|s| !s.via_container);
	let injects_param = if any_from_injects {
		format_ident!("injects")
	} else {
		format_ident!("_injects")
	};

	let inject_vars: Vec<_> = injects_spec
		.iter()
		.enumerate()
		.map(|(idx, _)| format_ident!("__mur_inject_{idx}"))
		.collect();

	// Determine whether each injected field's original type was Arc-wrapped.
	// For named fields we look up by ident; for unnamed (tuple struct) fields we
	// look up by the stored `original_field_idx`.
	let field_types_are_arc: Vec<bool> = injects_spec
		.iter()
		.map(|spec| {
			let field = if is_tuple_struct {
				input.fields.iter().nth(spec.original_field_idx)
			} else {
				input
					.fields
					.iter()
					.find(|f| f.ident.as_ref() == Some(&spec.field_name))
			};
			field
				.map(|f| {
					if extract_arc_inner(&f.ty).is_some() {
						return true;
					}
					if let Some(inner) = extract_option_inner(&f.ty)
						&& extract_arc_inner(inner).is_some()
					{
						return true;
					}
					false
				})
				.unwrap_or(false)
		})
		.collect();

	let inject_lets: Vec<TokenStream> = injects_spec
		.iter()
		.zip(inject_vars.iter())
		.zip(field_types_are_arc.iter())
		.map(|((spec, var), field_is_arc)| {
			let ty = &spec.ty;
			if spec.via_container {
				if spec.optional {
					if *field_is_arc {
						quote! { let #var = _container.get::<#ty>(); }
					} else {
						quote! { let #var = _container.get::<#ty>().map(|arc| (*arc).clone()); }
					}
				} else if *field_is_arc {
					quote! { let #var = _container.get_required::<#ty>(); }
				} else {
					quote! { let #var = (*_container.get_required::<#ty>()).clone(); }
				}
			} else if spec.optional {
				quote! { let #var = #injects_param.get::<#ty>(); }
			} else {
				quote! { let #var = #injects_param.get_required::<#ty>(); }
			}
		})
		.collect();

	// For tuple structs, any field that is NOT Arc-wrapped is an external type that the
	// container cannot produce (e.g. r2d2::Pool, PgConnection, etc.).
	// In that case we skip auto-injection entirely and generate a factory that
	// panics, so the type still compiles but must be registered manually.
	let tuple_has_external_fields =
		is_tuple_struct && field_types_are_arc.iter().any(|&is_arc| !is_arc);

	let create_expr = if tuple_has_external_fields {
		quote! {
			panic!(concat!(
				"`",
				stringify!(#struct_name),
				"` wraps an external type and must be registered manually \
				 (e.g. `container.register(", stringify!(#struct_name), "(value))`)."
			))
		}
	} else if is_tuple_struct {
		// All fields are Arc<T> — auto-inject from container.
		quote! { Self(#(#inject_vars),*) }
	} else if has_skipped_fields {
		quote! { Self::new(#(#inject_vars),*) }
	} else {
		let field_names: Vec<_> = injects_spec.iter().map(|s| &s.field_name).collect();
		quote! {
			Self {
				#(#field_names: #inject_vars),*
			}
		}
	};

	// When the tuple struct wraps external fields, clear inject_lets so the
	// generated `create` body only contains the panic — no get_required calls
	// that would fail to compile because the external type is not MurService.
	let inject_lets = if tuple_has_external_fields {
		vec![]
	} else {
		inject_lets
	};

	let required_container_deps: Vec<TokenStream> = if tuple_has_external_fields {
		vec![]
	} else {
		injects_spec
			.iter()
			.filter(|spec| spec.via_container && !spec.optional)
			.map(|spec| {
				let ty = &spec.ty;
				quote!(std::any::TypeId::of::<#ty>())
			})
			.collect()
	};

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

	(deps_impl, create_expr, inject_lets, injects_param)
}
