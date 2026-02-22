use serde_json::Value;

pub fn get_json_path<'a>(json: &'a Value, path: &str) -> Option<&'a Value> {
	let mut current = json;

	for part in path.split('.') {
		if let Some(bracket_pos) = part.find('[') {
			let key = &part[..bracket_pos];
			let index_str = &part[bracket_pos + 1..part.len() - 1];
			let index: usize = index_str.parse().ok()?;

			if !key.is_empty() {
				current = current.get(key)?;
			}

			current = current.get(index)?;
		} else {
			current = current.get(part)?;
		}
	}

	Some(current)
}
