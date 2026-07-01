use crate::generators::{ProjectGenerator, TemplateTypeEnum};
use cliclack::select;

pub fn execute(
	project_name: String,
	mut template_type: Option<TemplateTypeEnum>,
) -> Result<(), Box<dyn std::error::Error>> {
	if project_name.is_empty() {
		return Err("Project name must not be empty".into());
	}

	if std::path::Path::new(&project_name).exists() {
		return Err(format!("Directory '{}' already exists", project_name).into());
	}

	if template_type.is_none() {
		template_type = Some(
			select("Which template would you like to use?")
				.item(TemplateTypeEnum::Starter, "Starter", "Simple modular structure")
				.item(
					TemplateTypeEnum::Basic,
					"Basic",
					"Includes controllers and services",
				)
				.item(
					TemplateTypeEnum::Full,
					"Full",
					"Complete setup with example modules",
				)
				.interact()?,
		);
	}

	let generator = ProjectGenerator::new(project_name, template_type.unwrap());
	generator.generate()?;
	Ok(())
}
