pub struct MurDeflateEncoder {
	level: u32,
}

impl MurDeflateEncoder {
	pub fn new(level: u32) -> Self {
		Self { level }
	}

	pub fn compress(&self, data: &[u8]) -> Option<Vec<u8>> {
		Self::deflate_compress(data, self.level)
	}

	pub fn deflate_compress(data: &[u8], _level: u32) -> Option<Vec<u8>> {
		let mut output = Vec::with_capacity(data.len() + (data.len() / 65535 + 1) * 5 + 4);
		let chunks: Vec<&[u8]> = data.chunks(65535).collect();
		let num_chunks = chunks.len();

		for (i, chunk) in chunks.iter().enumerate() {
			let is_final = i == num_chunks - 1;
			output.push(if is_final { 0x01 } else { 0x00 });

			let len = chunk.len() as u16;
			output.push((len & 0xff) as u8);
			output.push((len >> 8) as u8);

			let nlen = !len;
			output.push((nlen & 0xff) as u8);
			output.push((nlen >> 8) as u8);
			output.extend_from_slice(chunk);
		}

		Some(output)
	}
}
