use std::any::TypeId;

pub trait MurDependencies {
	fn dependencies() -> &'static [TypeId] {
		&[]
	}
}
