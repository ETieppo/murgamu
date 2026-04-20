pub mod project_generator;
use clap::ValueEnum;
pub use project_generator::ProjectGenerator;

#[derive(Clone, ValueEnum, PartialEq, Eq)]
pub enum TemplateTypeEnum {
	Basic,
	Starter,
	Full,
}
