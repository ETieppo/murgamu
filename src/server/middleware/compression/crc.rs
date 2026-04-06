pub struct MurCrc;

impl MurCrc {
  pub fn crc32(data: &[u8]) -> u32 {
		const CRC32_TABLE: [u32; 256] = MurCrc::generate_crc32_table();

		let mut crc: u32 = 0xffffffff;
		for byte in data {
			let index = ((crc ^ (*byte as u32)) & 0xff) as usize;
			crc = CRC32_TABLE[index] ^ (crc >> 8);
		}
		!crc
	}

	pub const fn generate_crc32_table() -> [u32; 256] {
		let mut table = [0u32; 256];
		let mut i = 0;

		while i < 256 {
			let mut crc = i as u32;
			let mut j = 0;

			while j < 8 {
				if crc & 1 != 0 {
					crc = 0xedb88320 ^ (crc >> 1);
				} else {
					crc >>= 1;
				}
				j += 1;
			}
			table[i] = crc;
			i += 1;
		}
		table
	}
}
