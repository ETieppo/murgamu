pub fn normalize_path(path: &str) -> String {
	let path = path.trim();
	if path.is_empty() || path == "/" {
		return String::from("/");
	}
	let path = if path.starts_with('/') {
		path.to_string()
	} else {
		format!("/{}", path)
	};
	if path.ends_with('/') && path.len() > 1 {
		path.trim_end_matches('/').to_string()
	} else {
		path
	}
}
