use crate::middleware::compression::{crc::MurCrc, deflate::MurDeflateEncoder};

pub struct MurGzipEncoder {
	level: u32,
}

impl MurGzipEncoder {
	pub fn new(level: u32) -> Self {
		Self { level }
	}

	pub fn compress(&self, data: &[u8]) -> Option<Vec<u8>> {
		let mut output = Vec::with_capacity(data.len());
		output.extend_from_slice(&[0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff]);

		let compressed = MurDeflateEncoder::deflate_compress(data, self.level)?;
		output.extend_from_slice(&compressed);

		let crc = MurCrc::crc32(data);
		output.extend_from_slice(&crc.to_le_bytes());

		let size = data.len() as u32;
		output.extend_from_slice(&size.to_le_bytes());

		Some(output)
	}
}
