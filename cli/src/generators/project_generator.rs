use crate::templates::*;
use std::fs;
use std::path::Path;

const CORE_VERSION: &str = env!("MURGAMU_CORE_VERSION");

pub struct ProjectGenerator {
	project_name: String,
}

impl ProjectGenerator {
	pub fn new(project_name: String) -> Self {
		Self { project_name }
	}

	pub fn generate(&self) -> Result<(), Box<dyn std::error::Error>> {
		self.gen_dir()?;
		self.gen_files()?;

		println!("\n✨ '{}' successfully created!\n", self.project_name);
		println!("🚀 Next steps:");
		println!("   cd {}", self.project_name);
		println!("   mur dev\n");
		Ok(())
	}

	fn gen_dir(&self) -> Result<(), Box<dyn std::error::Error>> {
		let base = Path::new(&self.project_name);

		fs::create_dir_all(base)?;
		fs::create_dir_all(base.join("src"))?;
		fs::create_dir_all(base.join("src/mods"))?;
		fs::create_dir_all(base.join("src/mods/app"))?;
		fs::create_dir_all(base.join("src/mods/app/models"))?;
		Ok(())
	}

	fn gen_files(&self) -> Result<(), Box<dyn std::error::Error>> {
		let root_path = Path::new(&self.project_name);
		let modules_path = Path::new(&self.project_name).join("src/mods");
		let app_path = modules_path.join("app");

		fs::write(
			root_path.join("Cargo.toml"),
			CARGO_TEMPLATE
				.replace("{{project_name}}", &self.project_name)
				.replace("{{murgamu_version}}", CORE_VERSION),
		)?;
		fs::write(modules_path.join("mod.rs"), MODULES_MOD_TEMPLATE)?;
		fs::write(root_path.join(".rustfmt.toml"), FORMATTER_TEMPLATE)?;
		fs::write(root_path.join("src/main.rs"), MAIN_TEMPLATE)?;
		fs::write(app_path.join("app_controller.rs"), CONTROLLER_TEMPLATE)?;
		fs::write(app_path.join("app_service.rs"), SERVICE_TEMPLATE)?;
		fs::write(app_path.join("mod.rs"), APP_MOD_CONTENT_TEMPLATE)?;
		fs::write(app_path.join("models/mod.rs"), MODELS_MOD_TEMPLATE)?;
		fs::write(app_path.join("models/user_props.rs"), USER_PROPS_TEMPLATE)?;
		Ok(())
	}
}
