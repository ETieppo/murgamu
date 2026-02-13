// TODO: create tests
use super::mur_codec::MurCodec;
use crate::utils::mur_fmt::MurFmt;

#[test]
fn test_slugify() {
	assert_eq!(MurFmt::mur_slugify("Hello World"), "hello-world");
	assert_eq!(MurFmt::mur_slugify("Test  123"), "test-123");
	assert_eq!(MurFmt::mur_slugify("Special@#$Chars"), "specialchars");
}

#[test]
fn test_truncate() {
	assert_eq!(MurFmt::mur_truncate("hello", 10), "hello");
	assert_eq!(MurFmt::mur_truncate("hello world", 8), "hello...");
	assert_eq!(MurFmt::mur_truncate("hi", 2), "hi");
}

#[test]
fn test_base64_encode() {
	assert_eq!(MurCodec::base64_encode(b"hello"), "aGVsbG8=");
	assert_eq!(MurCodec::base64_encode(b"world!"), "d29ybGQh");
}

#[test]
fn test_decode() {
	assert_eq!(
		MurCodec::base64_decode("aGVsbG8=").unwrap(),
		b"hello".to_vec()
	);
	assert_eq!(
		MurCodec::base64_decode("d29ybGQh").unwrap(),
		b"world!".to_vec()
	);
}
