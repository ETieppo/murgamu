#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MurCompressionAlgorithm {
	Gzip,
	Brotli,
	Deflate,
	Identity,
}

impl std::fmt::Display for MurCompressionAlgorithm {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl MurCompressionAlgorithm {
	pub fn as_str(&self) -> &'static str {
		match self {
			MurCompressionAlgorithm::Gzip => "gzip",
			MurCompressionAlgorithm::Brotli => "br",
			MurCompressionAlgorithm::Deflate => "deflate",
			MurCompressionAlgorithm::Identity => "identity",
		}
	}

	pub fn from_accept_encoding(value: &str) -> Vec<(MurCompressionAlgorithm, f32)> {
		let mut algorithms = Vec::new();

		for part in value.split(',') {
			let part = part.trim();
			let (algo_str, quality) = if let Some(pos) = part.find(";q=") {
				let (algo, q) = part.split_at(pos);
				let q_val = q[3..].parse::<f32>().unwrap_or(1.0);
				(algo.trim(), q_val)
			} else {
				(part, 1.0)
			};

			let algo = match algo_str.to_lowercase().as_str() {
				"gzip" | "x-gzip" => Some(MurCompressionAlgorithm::Gzip),
				"br" => Some(MurCompressionAlgorithm::Brotli),
				"deflate" => Some(MurCompressionAlgorithm::Deflate),
				"identity" => Some(MurCompressionAlgorithm::Identity),
				"*" => Some(MurCompressionAlgorithm::Gzip),
				_ => None,
			};

			if let Some(a) = algo {
				algorithms.push((a, quality));
			}
		}

		algorithms.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
		algorithms
	}
}
