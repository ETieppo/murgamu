use super::*;
use hyper::body::Bytes;

#[test]
fn test_parse_boundary() {
	let content_type = "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW";
	let boundary = MurMultipartUtils::parse_boundary(content_type).unwrap();
	assert_eq!(boundary, "----WebKitFormBoundary7MA4YWxkTrZu0gW");

	let content_type = r#"multipart/form-data; boundary="my-boundary""#;
	let boundary = MurMultipartUtils::parse_boundary(content_type).unwrap();
	assert_eq!(boundary, "my-boundary");
}

#[test]
fn test_parse_boundary_missing() {
	let content_type = "multipart/form-data";
	assert!(MurMultipartUtils::parse_boundary(content_type).is_err());
}

#[test]
fn test_parse_boundary_wrong_content_type() {
	let content_type = "application/json";
	assert!(MurMultipartUtils::parse_boundary(content_type).is_err());
}

#[test]
fn test_sanitize_filename() {
	assert_eq!(MurMultipartUtils::sanitize_filename("hello.txt"), "hello.txt");
	assert_eq!(MurMultipartUtils::sanitize_filename("hello world.txt"), "hello world.txt");
	assert_eq!(MurMultipartUtils::sanitize_filename("../../../etc/passwd"), "etcpasswd");
	assert_eq!(MurMultipartUtils::sanitize_filename("file<>:\"|?*.txt"), "file.txt");
	assert_eq!(MurMultipartUtils::sanitize_filename(""), "unnamed_file");
	assert_eq!(MurMultipartUtils::sanitize_filename("..."), "unnamed_file");
}

#[test]
fn test_extract_extension() {
	assert_eq!(MurMultipartUtils::extract_extension("file.txt"), Some("txt".to_string()));
	assert_eq!(MurMultipartUtils::extract_extension("file.TXT"), Some("txt".to_string()));
	assert_eq!(MurMultipartUtils::extract_extension("file.tar.gz"), Some("gz".to_string()));
	assert_eq!(MurMultipartUtils::extract_extension("file"), None);
	assert_eq!(MurMultipartUtils::extract_extension(".hidden"), None);
	assert_eq!(MurMultipartUtils::extract_extension(".hidden.txt"), Some("txt".to_string()));
}

#[test]
fn test_uploaded_file() {
	let file = MurUploadedFile::new(
		"test.txt".to_string(),
		"text/plain".to_string(),
		Bytes::from("Hello, World!"),
		"document".to_string(),
	);

	assert_eq!(file.filename(), "test.txt");
	assert_eq!(file.sanitized_filename(), "test.txt");
	assert_eq!(file.content_type(), "text/plain");
	assert_eq!(file.extension(), Some("txt"));
	assert_eq!(file.size(), 13);
	assert_eq!(file.field_name(), "document");
	assert!(file.is_text());
	assert!(!file.is_image());
	assert!(file.has_extension("txt"));
	assert!(!file.has_extension("pdf"));
}

#[test]
fn test_uploaded_file_extensions() {
	let file = MurUploadedFile::new(
		"photo.jpg".to_string(),
		"image/jpeg".to_string(),
		Bytes::from(vec![0xFF, 0xD8, 0xFF]),
		"avatar".to_string(),
	);

	assert!(file.is_image());
	assert!(file.has_extension("jpg"));
	assert!(file.has_extension_in(&["jpg", "png", "gif"]));
	assert!(!file.has_extension_in(&["pdf", "doc"]));
}

#[test]
fn test_multipart_config() {
	let config = MurMultipartConfig::new()
		.max_body_size(100 * 1024 * 1024)
		.max_file_size(50 * 1024 * 1024)
		.max_files(10)
		.allowed_extensions(["jpg", "png"]);

	assert_eq!(config.max_body_size, 100 * 1024 * 1024);
	assert_eq!(config.max_file_size, 50 * 1024 * 1024);
	assert_eq!(config.max_files, 10);
	assert_eq!(config.allowed_extensions, vec!["jpg", "png"]);
}

#[test]
fn test_multipart_config_presets() {
	let permissive = MurMultipartConfig::permissive();
	assert_eq!(permissive.max_body_size, 500 * 1024 * 1024);

	let strict = MurMultipartConfig::strict();
	assert_eq!(strict.max_body_size, 5 * 1024 * 1024);

	let images = MurMultipartConfig::images_only();
	assert!(images.allowed_extensions.contains(&"jpg".to_string()));
	assert!(images
		.allowed_mime_types
		.contains(&"image/jpeg".to_string()));
}

#[test]
fn test_form_field() {
	let text_field = MurFormField::Text {
		name: "username".to_string(),
		value: "alice".to_string(),
	};

	assert!(text_field.is_text());
	assert!(!text_field.is_file());
	assert_eq!(text_field.name(), "username");
	assert_eq!(text_field.as_text(), Some("alice"));
	assert!(text_field.as_file().is_none());

	let file = MurUploadedFile::new(
		"doc.pdf".to_string(),
		"application/pdf".to_string(),
		Bytes::from(vec![]),
		"file".to_string(),
	);
	let file_field = MurFormField::File(file);

	assert!(!file_field.is_text());
	assert!(file_field.is_file());
	assert_eq!(file_field.name(), "file");
	assert!(file_field.as_file().is_some());
}

#[test]
fn test_multipart_empty() {
	let multipart = MurMultipart::empty();
	assert_eq!(multipart.fields_count(), 0);
	assert_eq!(multipart.files_count(), 0);
	assert!(!multipart.has_files());
}

#[test]
fn test_extract_header_param() {
	let header = r#"form-data; name="file"; filename="test.txt""#;
	assert_eq!(
		MurMultipartUtils::extract_header_param(header, "name"),
		Some("file".to_string())
	);
	assert_eq!(
		MurMultipartUtils::extract_header_param(header, "filename"),
		Some("test.txt".to_string())
	);
	assert_eq!(MurMultipartUtils::extract_header_param(header, "missing"), None);
}

#[test]
fn test_is_multipart_content_type() {
	assert!(MurMultipartUtils::mur_is_multipart_content_type(
		"multipart/form-data; boundary=abc"
	));
	assert!(MurMultipartUtils::mur_is_multipart_content_type(
		"MULTIPART/FORM-DATA; boundary=abc"
	));
	assert!(!MurMultipartUtils::mur_is_multipart_content_type("application/json"));
	assert!(!MurMultipartUtils::mur_is_multipart_content_type("text/plain"));
}

#[test]
fn test_unique_filename() {
	let file = MurUploadedFile::new(
		"document.pdf".to_string(),
		"application/pdf".to_string(),
		Bytes::from(vec![]),
		"file".to_string(),
	);

	let unique = file.unique_filename();
	assert!(unique.ends_with(".pdf"));
	assert!(unique.contains('_'));
}
