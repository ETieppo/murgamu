use std::process::Command;

pub fn clean_command(dir: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
	let cwd = match dir {
		Some(d) => std::path::PathBuf::from(d).canonicalize()?,
		None => std::env::current_dir()?,
	};

	Command::new("cargo").arg("run").current_dir(cwd).spawn()?;
	Ok(())
}
