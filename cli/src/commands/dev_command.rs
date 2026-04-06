use std::{path::PathBuf, process::Command};

pub fn dev_command(dir_to_execute: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
	let mut cmd = Command::new("cargo");
	cmd.args(["watch", "-x", "run"]);

	if let Some(dir) = dir_to_execute {
		let abs = PathBuf::from(dir).canonicalize()?;
		cmd.current_dir(abs);
	}

	cmd.spawn()?.wait()?;
	Ok(())
}
