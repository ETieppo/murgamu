pub mod algorithm;
pub mod mur_compression;
pub mod config;
pub mod crc;
pub mod deflate;
pub mod gzip;
pub mod level;

pub use algorithm::MurCompressionAlgorithm;
pub use mur_compression::MurCompression;
pub use config::MurCompressionConfig;
pub use crc::MurCrc;
pub use deflate::MurDeflateEncoder;
pub use gzip::MurGzipEncoder;
pub use level::MurCompressionLevel;

#[cfg(test)]
mod test;
