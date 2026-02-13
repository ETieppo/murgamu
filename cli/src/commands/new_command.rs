use crate::generators::ProjectGenerator;

pub fn execute(project_name: String) -> Result<(), Box<dyn std::error::Error>> {
	if project_name.is_empty() {
		return Err("Nome do projeto não pode ser vazio".into());
	}

	if std::path::Path::new(&project_name).exists() {
		return Err(format!("O diretório '{}' já existe", project_name).into());
	}

	let generator = ProjectGenerator::new(project_name);
	generator.generate()?;

	Ok(())
}
