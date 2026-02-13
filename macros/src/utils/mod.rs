mod analyze_parameter;
mod extract_generic_type;
mod is_constructor;
mod normalize_path;

pub use {
	analyze_parameter::analyze_parameter, extract_generic_type::extract_generic_type,
	is_constructor::is_constructor, normalize_path::normalize_path,
};

use crate::types::{ParamInfo, ParamKind};
