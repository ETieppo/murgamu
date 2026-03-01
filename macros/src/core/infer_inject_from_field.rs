use crate::core::{InjectSpec, extract_arc_inner, extract_option_inner};
use syn::ItemStruct;

pub fn infer_injects_from_fields(input: &ItemStruct) -> Vec<InjectSpec> {
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
