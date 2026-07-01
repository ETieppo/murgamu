use super::TemplateTypeEnum;
use crate::templates::*;
use std::fs;
use std::path::Path;

const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct ProjectGenerator {
	project_name: String,
	template_type: TemplateTypeEnum,
}

impl ProjectGenerator {
	pub fn new(project_name: String, template_type: TemplateTypeEnum) -> Self {
		Self {
			project_name,
			template_type,
		}
	}

	pub fn generate(&self) -> Result<(), Box<dyn std::error::Error>> {
		self.gen_dirs()?;
		self.gen_files()?;

		println!("\n 🕯️ '{}' successfully created!", self.project_name);
		println!("       - Next steps:");
		println!("         mur dev {}\n", self.project_name);
		println!("       - You can also");
		println!("         cd {} && mur dev\n", self.project_name);

		Ok(())
	}

	fn gen_dirs(&self) -> Result<(), Box<dyn std::error::Error>> {
		let base = Path::new(&self.project_name);

		match self.template_type {
			TemplateTypeEnum::Starter => {
				fs::create_dir_all(base.join("src/mods/app"))?;
			}
			TemplateTypeEnum::Full => {
				fs::create_dir_all(base.join("src/mods/app/models"))?;
				fs::create_dir_all(base.join("src/mods/auth"))?;
				fs::create_dir_all(base.join("src/mods/token/models"))?;
				fs::create_dir_all(base.join("src/mods/users"))?;
			}
			_ => {
				fs::create_dir_all(base.join("src/mods/"))?;
			}
		}

		Ok(())
	}

	fn gen_files(&self) -> Result<(), Box<dyn std::error::Error>> {
		let base = Path::new(&self.project_name);

		fs::write(
			base.join("Cargo.toml"),
			CARGO_TEMPLATE
				.replace("{{project_name}}", &self.project_name)
				.replace("{{murgamu_version}}", CORE_VERSION),
		)?;
		fs::write(base.join(".rustfmt.toml"), FORMATTER_TEMPLATE)?;
		fs::write(base.join(".env"), ENV_TEMPLATE)?;

		match self.template_type {
			TemplateTypeEnum::Starter => {
				fs::write(base.join("src/main.rs"), STARTER_MAIN_TEMPLATE)?;
				fs::write(base.join("src/mods/mod.rs"), STARTER_MODULES_MOD_TEMPLATE)?;
				fs::write(base.join("src/mods/app/mod.rs"), STARTER_APP_MOD_CONTENT_TEMPLATE)?;
				fs::write(
					base.join("src/mods/app/controller.rs"),
					STARTER_CONTROLLER_TEMPLATE,
				)?;
				fs::write(base.join("src/mods/app/service.rs"), STARTER_SERVICE_TEMPLATE)?;
			}
			TemplateTypeEnum::Full => {
				fs::write(base.join("src/main.rs"), FULL_MAIN_TEMPLATE)?;
				fs::write(base.join("src/mods/mod.rs"), FULL_MODULES_MOD_TEMPLATE)?;
				fs::write(base.join("src/mods/app/mod.rs"), FULL_APP_MOD_CONTENT_TEMPLATE)?;
				fs::write(base.join("src/mods/app/controller.rs"), FULL_CONTROLLER_TEMPLATE)?;
				fs::write(base.join("src/mods/app/service.rs"), FULL_SERVICE_TEMPLATE)?;
				fs::write(base.join("src/mods/app/models/mod.rs"), FULL_MODELS_MOD_TEMPLATE)?;
				fs::write(
					base.join("src/mods/app/models/user_props.rs"),
					FULL_USER_PROPS_TEMPLATE,
				)?;
				fs::write(base.join("src/mods/auth/mod.rs"), FULL_AUTH_MOD_TEMPLATE)?;
				fs::write(base.join("src/mods/auth/guard.rs"), FULL_AUTH_GUARD_TEMPLATE)?;
				fs::write(
					base.join("src/mods/auth/jwt_extraction_pipe.rs"),
					FULL_AUTH_PIPE_TEMPLATE,
				)?;
				fs::write(base.join("src/mods/token/mod.rs"), FULL_TOKEN_MOD_TEMPLATE)?;
				fs::write(
					base.join("src/mods/token/service.rs"),
					FULL_TOKEN_SERVICE_TEMPLATE,
				)?;
				fs::write(
					base.join("src/mods/token/models/mod.rs"),
					FULL_TOKEN_MODELS_MOD_TEMPLATE,
				)?;
				fs::write(base.join("src/mods/token/models/jwt.rs"), FULL_TOKEN_JWT_TEMPLATE)?;
				fs::write(base.join("src/mods/users/mod.rs"), FULL_USERS_MOD_TEMPLATE)?;
				fs::write(base.join("src/mods/users/role.rs"), FULL_USERS_ROLE_TEMPLATE)?;
			}
			TemplateTypeEnum::Basic => {
				fs::write(base.join("src/main.rs"), BASIC_MAIN_TEMPLATE)?;
				fs::write(base.join("src/mods/mod.rs"), BASIC_MODULES_MOD_TEMPLATE)?;
			}
		}

		Ok(())
	}
}
