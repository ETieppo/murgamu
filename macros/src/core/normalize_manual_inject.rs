use crate::core::{extract_arc_inner, extract_option_inner};
use syn::Type;

#[derive(Clone)]
pub struct InjectSpec {
	pub(crate) ty: Type,
	pub(crate) optional: bool,
	pub(crate) via_container: bool,
}

pub fn normalize_manual_inject(ty: &Type) -> InjectSpec {
	if let Some(opt_inner) = extract_option_inner(ty)
		&& let Some(arc_inner) = extract_arc_inner(opt_inner)
	{
		return InjectSpec {
			ty: arc_inner.clone(),
			optional: true,
			via_container: true,
		};
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
