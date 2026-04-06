use crate::core::{InjectSpec, extract_arc_inner, extract_option_inner};
use quote::format_ident;
use syn::{Attribute, Field, ItemStruct};

pub fn infer_injects_from_fields(input: &ItemStruct) -> Vec<InjectSpec> {
	input
		.fields
		.iter()
		.enumerate()
		.filter_map(|(idx, field)| {
			if is_skipped_field(field) {
				return None;
			}

			// Named fields use their ident; unnamed (tuple struct) fields get a
			// synthetic ident so the rest of the pipeline can treat them uniformly.
			let field_name = match field.ident.as_ref() {
				Some(ident) => ident.clone(),
				None => format_ident!("__mur_f{}", idx),
			};

			if let Some(opt_inner) = extract_option_inner(&field.ty) {
				if let Some(arc_inner) = extract_arc_inner(opt_inner) {
					return Some(InjectSpec {
						ty: arc_inner.clone(),
						field_name,
						original_field_idx: idx,
						optional: true,
						via_container: true,
					});
				}
				return Some(InjectSpec {
					ty: opt_inner.clone(),
					field_name,
					original_field_idx: idx,
					optional: true,
					via_container: true,
				});
			}

			if let Some(arc_inner) = extract_arc_inner(&field.ty) {
				return Some(InjectSpec {
					ty: arc_inner.clone(),
					field_name,
					original_field_idx: idx,
					optional: false,
					via_container: true,
				});
			}

			Some(InjectSpec {
				ty: field.ty.clone(),
				field_name,
				original_field_idx: idx,
				optional: false,
				via_container: true,
			})
		})
		.collect()
}

pub fn is_skipped_field(field: &Field) -> bool {
	let name_starts_with_underscore = field
		.ident
		.as_ref()
		.map(|n| n.to_string().starts_with('_'))
		.unwrap_or(false);

	name_starts_with_underscore || has_inject_skip(&field.attrs)
}

fn has_inject_skip(attrs: &[Attribute]) -> bool {
	attrs.iter().any(|attr| {
		if !attr.path().is_ident("inject") {
			return false;
		}
		let mut found = false;
		let _ = attr.parse_nested_meta(|meta| {
			if meta.path.is_ident("skip") {
				found = true;
			}
			Ok(())
		});
		found
	})
}
