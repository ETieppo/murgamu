pub fn parse_duration(s: &str) -> Option<std::time::Duration> {
	let s = s.trim().to_lowercase();

	if let Ok(secs) = s.parse::<u64>() {
		return Some(std::time::Duration::from_secs(secs));
	}

	let (num_str, multiplier) = if let Some(n) = s.strip_suffix("ms") {
		(n, 1u64)
	} else if let Some(n) = s.strip_suffix('s') {
		(n, 1000)
	} else if let Some(n) = s.strip_suffix('m') {
		(n, 60 * 1000)
	} else if let Some(n) = s.strip_suffix('h') {
		(n, 60 * 60 * 1000)
	} else if let Some(n) = s.strip_suffix('d') {
		(n, 24 * 60 * 60 * 1000)
	} else {
		return None;
	};

	num_str
		.trim()
		.parse::<u64>()
		.ok()
		.map(|n| std::time::Duration::from_millis(n * multiplier))
}

pub fn parse_size(s: &str) -> Option<u64> {
	let s = s.trim().to_uppercase();

	if let Ok(bytes) = s.parse::<u64>() {
		return Some(bytes);
	}

	let (num_str, multiplier) = if let Some(n) = s.strip_suffix("GB") {
		(n, 1024 * 1024 * 1024)
	} else if let Some(n) = s.strip_suffix("MB") {
		(n, 1024 * 1024)
	} else if let Some(n) = s.strip_suffix("KB") {
		(n, 1024)
	} else if let Some(n) = s.strip_suffix('B') {
		(n, 1)
	} else {
		return None;
	};

	num_str.trim().parse::<u64>().ok().map(|n| n * multiplier)
}
