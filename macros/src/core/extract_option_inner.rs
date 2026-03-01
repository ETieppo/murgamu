use crate::core::extract_arc_inner;
use syn::{GenericArgument, PathArguments, Type};

pub fn extract_option_inner(ty: &Type) -> Option<&Type> {
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
		GenericArgument::Type(inner) => extract_arc_inner(inner),
		_ => None,
	}
}
