use crate::core::{extract_arc_inner, extract_option_inner};
use syn::{Ident, Type};

#[derive(Clone)]
pub struct InjectSpec {
	pub(crate) ty: Type,
	pub(crate) field_name: syn::Ident,
	/// Index of this field in the original struct definition (used for unnamed/tuple fields).
	pub(crate) original_field_idx: usize,
	pub(crate) optional: bool,
	pub(crate) via_container: bool,
}

pub fn normalize_manual_inject(ty: &Type, field_name: Ident) -> InjectSpec {
	if let Some(opt_inner) = extract_option_inner(ty)
		&& let Some(arc_inner) = extract_arc_inner(opt_inner)
	{
		return InjectSpec {
			ty: arc_inner.clone(),
			field_name,
			original_field_idx: 0,
			optional: true,
			via_container: true,
		};
	}

	if let Some(arc_inner) = extract_arc_inner(ty) {
		return InjectSpec {
			ty: arc_inner.clone(),
			field_name,
			original_field_idx: 0,
			optional: false,
			via_container: true,
		};
	}

	InjectSpec {
		ty: ty.clone(),
		field_name,
		original_field_idx: 0,
		optional: false,
		via_container: false,
	}
}
