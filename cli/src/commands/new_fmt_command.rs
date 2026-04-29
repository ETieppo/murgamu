use crate::templates::FORMATTER_TEMPLATE;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const FORMATTER_FILENAME: &str = ".rustfmt.toml";

pub fn new_fmt_command(
	dir: Option<String>,
	unstable: bool,
	overwrite: bool,
	merge: bool,
) -> Result<(), Box<dyn std::error::Error>> {
	let dir = dir.unwrap_or_default();
	let display_dir = if dir.is_empty() { "." } else { dir.as_str() };
	let fmt_path = if dir.is_empty() {
		FORMATTER_FILENAME.to_string()
	} else {
		format!("{}/{}", dir, FORMATTER_FILENAME)
	};
	let exists = Path::new(&fmt_path).exists();

	if exists && !overwrite && !merge {
		return Err(format!(
			"{} already exists at {}; use --overwrite to replace or --merge to merge",
			FORMATTER_FILENAME, display_dir
		)
		.into());
	}

	if overwrite && merge {
		return Err("--overwrite and --merge are mutually exclusive".into());
	}

	if exists && merge {
		merge_formatter(&fmt_path, unstable)?;
	} else {
		let content = render_template(unstable);
		gen_new_formatter(&content, &fmt_path)?;
	}

	Ok(())
}

fn render_template(unstable: bool) -> String {
	if unstable {
		uncomment_template(FORMATTER_TEMPLATE)
	} else {
		FORMATTER_TEMPLATE.to_string()
	}
}

fn uncomment_template(template: &str) -> String {
	template
		.lines()
		.map(|line| {
			let trimmed_start = line.trim_start();
			let indent_len = line.len() - trimmed_start.len();
			if let Some(rest) = trimmed_start.strip_prefix("# ") {
				format!("{}{}", &line[..indent_len], rest)
			} else if let Some(rest) = trimmed_start.strip_prefix('#') {
				format!("{}{}", &line[..indent_len], rest)
			} else {
				line.to_string()
			}
		})
		.collect::<Vec<_>>()
		.join("\n")
}

fn extract_key(line: &str) -> Option<&str> {
	let mut s = line.trim_start();
	if let Some(rest) = s.strip_prefix('#') {
		s = rest.trim_start();
	}
	let eq = s.find('=')?;
	let key = s[..eq].trim();
	if key.is_empty() { None } else { Some(key) }
}

fn merge_formatter(fmt_path: &str, unstable: bool) -> Result<(), std::io::Error> {
	let existing = fs::read_to_string(fmt_path)?;
	let existing_keys: HashSet<String> = existing
		.lines()
		.filter(|l| !l.trim_start().starts_with('#'))
		.filter_map(extract_key)
		.map(|k| k.to_string())
		.collect();
	let rendered = render_template(unstable);
	let new_lines: Vec<&str> = rendered
		.lines()
		.filter(|line| match extract_key(line) {
			Some(key) => !existing_keys.contains(key),
			None => true,
		})
		.collect();
	let has_real_content = new_lines.iter().any(|l| extract_key(l).is_some());

	if !has_real_content {
		return Ok(());
	}

	let start = new_lines
		.iter()
		.position(|l| extract_key(l).is_some())
		.unwrap_or(0);
	let end = new_lines
		.iter()
		.rposition(|l| extract_key(l).is_some())
		.map(|i| i + 1)
		.unwrap_or(new_lines.len());
	let to_append = &new_lines[start..end];
	let mut output = existing;

	if !output.ends_with('\n') {
		output.push('\n');
	}
	output.push('\n');
	output.push_str(&to_append.join("\n"));
	output.push('\n');
	fs::write(fmt_path, output)
}

fn gen_new_formatter(content: &str, fmt_path: &str) -> Result<(), std::io::Error> {
	fs::write(fmt_path, content)
}
