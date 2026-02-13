use std::fs;
use std::path::Path;

fn main() {
	let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
	let root_cargo_path = Path::new(&manifest_dir).join("../Cargo.toml");
	let version = if root_cargo_path.exists() {
		let cargo_toml =
			fs::read_to_string(root_cargo_path).expect("Error reading murgamu core version!");

		cargo_toml
			.lines()
			.find(|line| line.starts_with("version ="))
			.and_then(|line| line.split('"').nth(1))
			.expect("Not found version")
			.to_string()
	} else {
		std::env::var("CARGO_PKG_VERSION").unwrap()
	};

	println!("cargo:rustc-env=MURGAMU_CORE_VERSION={}", version);
	println!("cargo:rerun-if-changed=../Cargo.toml");
}
