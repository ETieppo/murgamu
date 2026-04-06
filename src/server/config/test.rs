use super::*;

#[test]
fn test_mur_env_or() {
	let value = mur_env_or("NONEXISTENT_KEY_12345", "default");
	assert_eq!(value, "default");
}

#[test]
fn test_mur_env_parse_or() {
	let value: u16 = mur_env_parse_or("NONEXISTENT_KEY_12345", 3000);
	assert_eq!(value, 3000);
}

#[test]
fn test_mur_env_is_true() {
	assert!(!mur_env_is_true("NONEXISTENT_KEY_12345"));
}
