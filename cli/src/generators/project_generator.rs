use crate::templates::*;
use std::fs;
use std::path::Path;

const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct ProjectGenerator {
	project_name: String,
}

impl ProjectGenerator {
	pub fn new(project_name: String) -> Self {
		Self { project_name }
	}

	pub fn generate(&self) -> Result<(), Box<dyn std::error::Error>> {
		self.gen_dirs()?;
		self.gen_files()?;

		println!("\n🕯️ '{}' successfully created!\n", self.project_name);
		println!("Next steps:");
		println!("cd {}", self.project_name);
		println!("mur dev\n");
		Ok(())
	}

	fn gen_dirs(&self) -> Result<(), Box<dyn std::error::Error>> {
		let base = Path::new(&self.project_name);
		fs::create_dir_all(base.join("src/mods/app/models"))?;
		fs::create_dir_all(base.join("src/mods/auth"))?;
		fs::create_dir_all(base.join("src/mods/token/models"))?;
		fs::create_dir_all(base.join("src/mods/users"))?;
		Ok(())
	}

	fn gen_files(&self) -> Result<(), Box<dyn std::error::Error>> {
		let base = Path::new(&self.project_name);

		// root
		fs::write(
			base.join("Cargo.toml"),
			CARGO_TEMPLATE
				.replace("{{project_name}}", &self.project_name)
				.replace("{{murgamu_version}}", CORE_VERSION),
		)?;
		fs::write(base.join(".rustfmt.toml"), FORMATTER_TEMPLATE)?;

		// src/
		fs::write(base.join("src/main.rs"), MAIN_TEMPLATE)?;

		// src/mods/
		fs::write(base.join("src/mods/mod.rs"), MODULES_MOD_TEMPLATE)?;

		// src/mods/app/
		fs::write(base.join("src/mods/app/mod.rs"), APP_MOD_CONTENT_TEMPLATE)?;
		fs::write(base.join("src/mods/app/app_controller.rs"), CONTROLLER_TEMPLATE)?;
		fs::write(base.join("src/mods/app/app_service.rs"), SERVICE_TEMPLATE)?;
		fs::write(base.join("src/mods/app/models/mod.rs"), MODELS_MOD_TEMPLATE)?;
		fs::write(base.join("src/mods/app/models/user_props.rs"), USER_PROPS_TEMPLATE)?;

		// src/mods/auth/
		fs::write(base.join("src/mods/auth/mod.rs"), AUTH_MOD_TEMPLATE)?;
		fs::write(base.join("src/mods/auth/guard.rs"), AUTH_GUARD_TEMPLATE)?;
		fs::write(base.join("src/mods/auth/jwt_extraction_pipe.rs"), AUTH_PIPE_TEMPLATE)?;

		// src/mods/token/
		fs::write(base.join("src/mods/token/mod.rs"), TOKEN_MOD_TEMPLATE)?;
		fs::write(base.join("src/mods/token/service.rs"), TOKEN_SERVICE_TEMPLATE)?;
		fs::write(base.join("src/mods/token/models/mod.rs"), TOKEN_MODELS_MOD_TEMPLATE)?;
		fs::write(base.join("src/mods/token/models/jwt.rs"), TOKEN_JWT_TEMPLATE)?;

		// src/mods/users/
		fs::write(base.join("src/mods/users/mod.rs"), USERS_MOD_TEMPLATE)?;
		fs::write(base.join("src/mods/users/role.rs"), USERS_ROLE_TEMPLATE)?;

		Ok(())
	}
}
