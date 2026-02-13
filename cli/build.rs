// cli/build.rs
use std::fs;

fn main() {
    // Durante o build, o 'cargo' define o diretório atual como a pasta do projeto.
    // Então "../Cargo.toml" aponta para o root do Murgamü.
    let cargo_toml = fs::read_to_string("../Cargo.toml")
        .expect("Erro ao ler Cargo.toml durante a compilação");

    let version = cargo_toml
        .lines()
        .find(|line| line.starts_with("version ="))
        .and_then(|line| line.split('"').nth(1))
        .expect("Versão não encontrada");

    // Isso INJETA a versão no binário. Não precisa de arquivo no PC do usuário.
    println!("cargo:rustc-env=MURGAMU_CORE_VERSION={}", version);
}