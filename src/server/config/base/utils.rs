/// Parses a duration string into a [`std::time::Duration`].
///
/// Accepted formats:
///
/// | Input   | Meaning      |
/// |---------|-------------|
/// | `"30"`  | 30 seconds   |
/// | `"500ms"` | 500 milliseconds |
/// | `"30s"` | 30 seconds   |
/// | `"5m"`  | 5 minutes    |
/// | `"2h"`  | 2 hours      |
/// | `"1d"`  | 1 day        |
///
/// Returns `None` for unrecognised formats.
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

/// Parses a human-readable byte-size string into a raw byte count.
///
/// Accepted formats:
///
/// | Input    | Meaning     |
/// |----------|-------------|
/// | `"1024"` | 1 024 bytes |
/// | `"16KB"` | 16 384 bytes |
/// | `"8MB"`  | 8 388 608 bytes |
/// | `"2GB"`  | 2 147 483 648 bytes |
///
/// Returns `None` for unrecognised formats.
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
