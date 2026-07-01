pub mod algorithm;
pub mod config;
pub mod crc;
pub mod deflate;
pub mod gzip;
pub mod level;
pub mod mur_compression;

pub use algorithm::MurCompressionAlgorithm;
pub use config::MurCompressionConfig;
pub use crc::MurCrc;
pub use deflate::MurDeflateEncoder;
pub use gzip::MurGzipEncoder;
pub use level::MurCompressionLevel;
pub use mur_compression::MurCompression;

#[cfg(test)]
mod test;
