use syn::{ReturnType, Type};

pub fn is_constructor(return_type: &ReturnType) -> bool {
	match return_type {
		ReturnType::Default => false,
		ReturnType::Type(_, ty) => {
			if let Type::Path(type_path) = ty.as_ref() {
				type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Self"
			} else {
				false
			}
		}
	}
}
