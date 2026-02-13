// TODO: create tests
use super::*;

#[test]
fn test_compression_algorithm_from_accept_encoding() {
	let algos = MurCompressionAlgorithm::from_accept_encoding("gzip, deflate, br");
	assert_eq!(algos.len(), 3);
	assert_eq!(algos[0].0, MurCompressionAlgorithm::Gzip);

	let algos = MurCompressionAlgorithm::from_accept_encoding("br;q=1.0, gzip;q=0.8");
	assert_eq!(algos[0].0, MurCompressionAlgorithm::Brotli);
	assert_eq!(algos[1].0, MurCompressionAlgorithm::Gzip);
}

#[test]
fn test_compression_level() {
	assert_eq!(MurCompressionLevel::Fastest.gzip_level(), 1);
	assert_eq!(MurCompressionLevel::Fast.gzip_level(), 3);
	assert_eq!(MurCompressionLevel::Default.gzip_level(), 6);
	assert_eq!(MurCompressionLevel::Best.gzip_level(), 9);
	assert_eq!(MurCompressionLevel::Custom(5).gzip_level(), 5);
}

#[test]
fn test_crc32() {
	let data = b"123456789";
	let crc = MurCrc::crc32(data);
	assert_eq!(crc, 0xCBF43926);
}

#[test]
fn test_should_compress_content_type() {
	let compression = MurCompression::default();
	assert!(compression.should_compress_content_type(Some("text/html")));
	assert!(compression.should_compress_content_type(Some("application/json")));
	assert!(compression.should_compress_content_type(Some("text/css")));
	assert!(compression.should_compress_content_type(Some("application/javascript")));
	assert!(!compression.should_compress_content_type(Some("image/jpeg")));
	assert!(!compression.should_compress_content_type(Some("image/png")));
	assert!(!compression.should_compress_content_type(Some("video/mp4")));
}

// TODO:
// FIXME:
// #[test]
// fn test_select_algorithm() {
// 	let compression = MurCompression::gzip_only();
// 	assert_eq!(
// 		compression.select_algorithm(Some("gzip, deflate")),
// 		Some(MurCompressionAlgorithm::Gzip)
// 	);
// 	assert_eq!(compression.select_algorithm(Some("br")), None);

// 	let compression = MurCompression::new().gzip().brotli();
// 	assert_eq!(
// 		compression.select_algorithm(Some("br;q=1.0, gzip;q=0.5")),
// 		Some(MurCompressionAlgorithm::Brotli)
// 	);
// }

// #[test]
// fn test_gzip_compression() {
// 	let compression = MurCompression::gzip_only();
// 	let data = b"Hello, World! This is some test data for compression.";
// 	let compressed = compression.compress(data, MurCompressionAlgorithm::Gzip);
// 	assert!(compressed.is_some());

// 	let compressed = compressed.unwrap();
// 	assert_eq!(compressed[0], 0x1f);
// 	assert_eq!(compressed[1], 0x8b);
// }

// #[test]
// fn test_builder_pattern() {
// 	let compression = MurCompression::new()
// 		.gzip()
// 		.brotli()
// 		.level(MurCompressionLevel::Fast)
// 		.min_size(2048)
// 		.content_type("application/json")
// 		.exclude_type("text/plain");

// 	assert!(compression
// 		.config
// 		.algorithms
// 		.contains(&MurCompressionAlgorithm::Gzip));
// 	assert!(compression
// 		.config
// 		.algorithms
// 		.contains(&MurCompressionAlgorithm::Brotli));
// 	assert_eq!(compression.config.level, MurCompressionLevel::Fast);
// 	assert_eq!(compression.config.min_size, 2048);
// }
