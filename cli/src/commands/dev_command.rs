use std::process::Command;

pub fn dev_command() -> Result<(), Box<dyn std::error::Error>> {
	Command::new("cargo")
		.args(["watch", "-x", "run"])
		.spawn()?
		.wait()?;

	Ok(())
}
