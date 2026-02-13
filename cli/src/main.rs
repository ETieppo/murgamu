mod commands;
mod generators;
mod templates;

use crate::commands::dev_command;
use crate::commands::ensure_cargo_watch;
use crate::commands::execute;
use clap::ArgAction;
use clap::Parser;
use clap::Subcommand;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
	name = "mur",
	author = "E.Tieppo",
	version = VERSION,
	about = "🚀 Murgamü CLI",
	long_about = "Murgamü CLI is a command-line tool for creating and managing Murgamü projects.\nIt provides a modular architecture for building scalable web applications in Rust. You can use it for hot reload too",
	    disable_version_flag = true
)]
struct Cli {
	#[arg(short = 'v', long = "version", global = true, action = ArgAction::SetTrue)]
	version_flag: bool,
	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	#[command(
		about = "Create a new Murgamü project with modular structure",
		long_about = "Creates a new Murgamü project with a complete file structure including:\n  - Main application file\n  - Module structure (controllers, services, modules)\n  - Example implementations\n  - Cargo configuration"
	)]
	New {
		#[arg(value_name = "PROJECT_NAME")]
		name: String,
	},
	#[command(
		about = "Run project with hot reload",
		long_about = "Starts development mode using cargo-watch for automatic rebuild and restart"
	)]
	Dev {
		#[arg(value_name = "DIR")]
		dir: Option<String>,
	},
}

fn main() {
	let cli = Cli::parse();

	if cli.version_flag {
		println!("mur {}", VERSION);
		return;
	}

	let Some(command) = cli.command else {
		println!("use --help for info");
		return;
	};

	let result = match command {
		Commands::New { name } => execute(name),
		Commands::Dev { dir } => {
			ensure_cargo_watch().expect("[ERROR]: to ensure cargo watch");
			dev_command(dir)
		}
	};

	if let Err(e) = result {
		eprintln!("[ERROR]: {}", e);
		std::process::exit(1);
	}
}
