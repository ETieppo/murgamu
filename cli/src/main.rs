mod commands;
mod generators;
mod templates;

use crate::commands::clean_command;
use crate::commands::dev_command;
use crate::commands::execute;
use crate::commands::new_fmt_command;
use crate::generators::TemplateTypeEnum;
use clap::ArgAction;
use clap::Parser;
use clap::Subcommand;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
	name = "mur",
	author = "E.Tieppo",
	version = VERSION,
	about = "🕯️  Murgamü CLI",
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
		#[arg(long, action = ArgAction::SetTrue)]
		starter: bool,
		#[arg(long, action = ArgAction::SetTrue)]
		basic: bool,
		#[arg(long, action = ArgAction::SetTrue)]
		full: bool,
	},
	#[command(
		about = "Run project with hot reload",
		long_about = "Starts development mode with automatic rebuild and restart on file changes"
	)]
	Dev {
		#[arg(value_name = "DIR")]
		dir: Option<String>,
	},
	#[command(about = "Execute cargo clean")]
	Clean {
		#[arg(value_name = "DIR")]
		dir: Option<String>,
	},
	#[command(
		about = "Add .rustfmt.toml inside project",
		long_about = "Create .rustfmt.toml into project, the file will be created at actual directory if not agrgs was provided, otherwise inside provided dir"
	)]
	NewFmt {
		#[arg(value_name = "DIR")]
		dir: Option<String>,
		#[arg(long, action=ArgAction::SetTrue)]
		overwrite: bool,
		#[arg(long, action=ArgAction::SetTrue)]
		unstable: bool,
		#[arg(long, action=ArgAction::SetTrue)]
		merge: bool,
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
		Commands::New {
			name,
			starter,
			basic,
			full,
		} => {
			let template = if starter {
				Some(TemplateTypeEnum::Starter)
			} else if basic {
				Some(TemplateTypeEnum::Basic)
			} else if full {
				Some(TemplateTypeEnum::Full)
			} else {
				None
			};
			execute(name, template)
		}
		Commands::Dev { dir } => dev_command(dir),
		Commands::Clean { dir } => clean_command(dir),
		Commands::NewFmt {
			dir,
			unstable,
			overwrite,
			merge,
		} => new_fmt_command(dir, unstable, overwrite, merge),
	};

	if let Err(e) = result {
		eprintln!("[ERROR]: {}", e);
		std::process::exit(1);
	}
}
