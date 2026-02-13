pub const CARGO_TEMPLATE: &str = r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"

[dependencies]
murgamu = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
"#;
