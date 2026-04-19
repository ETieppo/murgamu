pub const CARGO_TEMPLATE: &str = r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2024"

[dependencies]
chrono = "0.4"
jsonwebtoken = "10"
murgamu = "{{murgamu_version}}"
serde = { version = "1", features = ["derive"] }
strum_macros = "0.28"
"#;
