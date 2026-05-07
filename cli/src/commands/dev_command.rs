use notify::{RecursiveMode, Watcher};
use std::process::{Child, Command};
use std::sync::mpsc;
use std::time::Duration;

pub fn dev_command(dir: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
	let cwd = match dir {
		Some(d) => std::path::PathBuf::from(d).canonicalize()?,
		None => std::env::current_dir()?,
	};
	let src_path = cwd.join("src");
	let cargo_path = cwd.join("Cargo.toml");
	let (tx, rx) = mpsc::channel::<()>();
	let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
		if let Ok(event) = res
			&& (event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove())
		{
			let _ = tx.send(());
		}
	})?;

	if src_path.exists() {
		watcher.watch(&src_path, RecursiveMode::Recursive)?;
	}
	if cargo_path.exists() {
		watcher.watch(&cargo_path, RecursiveMode::NonRecursive)?;
	}

	loop {
		let mut child = spawn_cargo_run(&cwd)?;

		loop {
			match rx.recv_timeout(Duration::from_millis(100)) {
				Ok(_) => {
					std::thread::sleep(Duration::from_millis(200));
					while rx.try_recv().is_ok() {}
					println!("\n[mur] File changed, restarting...\n");
					kill_child(&mut child);
					break;
				}
				Err(mpsc::RecvTimeoutError::Timeout) => {
					if let Ok(Some(_)) = child.try_wait() {
						println!("[mur] Process exited. Waiting for changes...");
						rx.recv()?;
						while rx.try_recv().is_ok() {}
						break;
					}
				}
				Err(mpsc::RecvTimeoutError::Disconnected) => {
					return Err("File watcher disconnected".into());
				}
			}
		}
	}
}

fn spawn_cargo_run(cwd: &std::path::Path) -> Result<Child, Box<dyn std::error::Error>> {
	Ok(Command::new("cargo").arg("run").current_dir(cwd).spawn()?)
}

fn kill_child(child: &mut Child) {
	let _ = child.kill();
	let _ = child.wait();
}
