pub struct MurFmt;
impl MurFmt {
	pub fn mur_slugify(s: &str) -> String {
		let mut result = String::new();
		let mut last_was_dash = true;

		for c in s.to_lowercase().chars() {
			if c.is_alphanumeric() {
				result.push(c);
				last_was_dash = false;
			} else if (c.is_whitespace() || c == '-' || c == '_') && !last_was_dash {
				result.push('-');
				last_was_dash = true;
			}
		}

		if result.ends_with('-') {
			result.pop();
		}

		result
	}
	pub fn mur_truncate(s: &str, max_len: usize) -> String {
		if s.len() <= max_len {
			s.to_string()
		} else if max_len <= 3 {
			s.chars().take(max_len).collect()
		} else {
			format!("{}...", s.chars().take(max_len - 3).collect::<String>())
		}
	}
}
