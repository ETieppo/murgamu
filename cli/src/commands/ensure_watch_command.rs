use std::process::{Command, Stdio};

pub fn ensure_cargo_watch() -> Result<(), Box<dyn std::error::Error>> {
	let installed = Command::new("cargo")
		.args(["watch", "--version"])
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.status()
		.map(|s| s.success())
		.unwrap_or(false);

	if installed {
		return Ok(());
	}

	println!("You do not have cargo-watch, installing...");

	let status = Command::new("cargo")
		.args(["install", "cargo-watch", "--locked", "--quiet"])
		.stdin(Stdio::null())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.status()?;

	if !status.success() {
		return Err("failed to install cargo-watch".into());
	}

	println!("cargo-watch ready!\n");
	Ok(())
}
