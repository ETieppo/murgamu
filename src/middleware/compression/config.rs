use crate::middleware::compression::{
	algorithm::MurCompressionAlgorithm, level::MurCompressionLevel,
};

#[derive(Debug, Clone)]
pub struct MurCompressionConfig {
	pub algorithms: Vec<MurCompressionAlgorithm>,
	pub level: MurCompressionLevel,
	pub min_size: usize,
	pub content_types: Vec<String>,
	pub exclude_types: Vec<String>,
	pub compress_without_accept_encoding: bool,
}

impl Default for MurCompressionConfig {
	fn default() -> Self {
		Self {
			algorithms: vec![
				MurCompressionAlgorithm::Gzip,
				MurCompressionAlgorithm::Brotli,
			],
			level: MurCompressionLevel::Default,
			min_size: 860,
			content_types: Vec::new(),
			exclude_types: vec![
				"image/jpeg".to_string(),
				"image/png".to_string(),
				"image/gif".to_string(),
				"image/webp".to_string(),
				"video/".to_string(),
				"audio/".to_string(),
			],
			compress_without_accept_encoding: false,
		}
	}
}
