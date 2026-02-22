#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MurCompressionLevel {
	Fastest,
	Fast,
	Best,
	Custom(u32),
	#[default]
	Default,
}

impl MurCompressionLevel {
	pub fn gzip_level(&self) -> u32 {
		match self {
			MurCompressionLevel::Fastest => 1,
			MurCompressionLevel::Fast => 3,
			MurCompressionLevel::Default => 6,
			MurCompressionLevel::Best => 9,
			MurCompressionLevel::Custom(level) => (*level).clamp(1, 9),
		}
	}

	pub fn brotli_level(&self) -> u32 {
		match self {
			MurCompressionLevel::Fastest => 1,
			MurCompressionLevel::Fast => 4,
			MurCompressionLevel::Default => 6,
			MurCompressionLevel::Best => 11,
			MurCompressionLevel::Custom(level) => (*level).min(11),
		}
	}
}
