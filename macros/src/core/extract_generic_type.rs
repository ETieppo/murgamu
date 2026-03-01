pub fn extract_generic_type(ty_str: &str, wrapper: &str) -> String {
	if let Some(start) = ty_str.find(&format!("{}<", wrapper)) {
		let start = start + wrapper.len() + 1;
		let mut depth = 1;
		let mut end = start;
		for (i, c) in ty_str[start..].chars().enumerate() {
			match c {
				'<' => depth += 1,
				'>' => {
					depth -= 1;
					if depth == 0 {
						end = start + i;
						break;
					}
				}
				_ => {}
			}
		}
		ty_str[start..end].to_string()
	} else {
		String::new()
	}
}
