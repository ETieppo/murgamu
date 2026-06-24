//! Minimal but correct DEFLATE (RFC 1951) encoder using LZ77 matching and the
//! fixed/static Huffman code tables (block type 01).
//!
//! This is intentionally dependency-free. It is not the smallest possible
//! output (it uses static Huffman, not dynamic), but it is spec-compliant and
//! decodable by any standard inflater (browsers, `gzip`, `flate2`, …), and it
//! actually compresses repetitive data instead of merely storing it.

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

	/// Compresses `data` into a single static-Huffman DEFLATE block.
	pub fn deflate_compress(data: &[u8], _level: u32) -> Option<Vec<u8>> {
		let mut writer = BitWriter::new(data.len() / 2 + 16);

		// Block header: BFINAL = 1, BTYPE = 01 (fixed Huffman). Both LSB-first.
		writer.write_bits(1, 1);
		writer.write_bits(0b01, 2);

		lz77_encode(data, &mut writer);

		// End-of-block symbol (256).
		write_litlen_symbol(&mut writer, 256);

		Some(writer.finish())
	}
}

// ---------------------------------------------------------------------------
// Bit writer
// ---------------------------------------------------------------------------

struct BitWriter {
	out: Vec<u8>,
	bitbuf: u64,
	nbits: u32,
}

impl BitWriter {
	fn new(cap: usize) -> Self {
		Self {
			out: Vec::with_capacity(cap),
			bitbuf: 0,
			nbits: 0,
		}
	}

	/// Writes the low `n` bits of `value`, least-significant bit first.
	/// Used for the block header and for length/distance "extra" bits.
	fn write_bits(&mut self, value: u32, n: u32) {
		debug_assert!(n <= 32);
		self.bitbuf |= ((value as u64) & ((1u64 << n) - 1)) << self.nbits;
		self.nbits += n;
		while self.nbits >= 8 {
			self.out.push((self.bitbuf & 0xff) as u8);
			self.bitbuf >>= 8;
			self.nbits -= 8;
		}
	}

	/// Writes a Huffman code of `n` bits, most-significant bit first, as
	/// required by RFC 1951 §3.1.1 for the code itself.
	fn write_code(&mut self, code: u32, n: u32) {
		for i in (0..n).rev() {
			self.write_bits((code >> i) & 1, 1);
		}
	}

	fn finish(mut self) -> Vec<u8> {
		if self.nbits > 0 {
			self.out.push((self.bitbuf & 0xff) as u8);
		}
		self.out
	}
}

// ---------------------------------------------------------------------------
// Static Huffman symbol emission
// ---------------------------------------------------------------------------

/// Emits a literal/length symbol using the fixed Huffman table (RFC 1951 §3.2.6).
fn write_litlen_symbol(w: &mut BitWriter, sym: u32) {
	match sym {
		0..=143 => w.write_code(0x30 + sym, 8),
		144..=255 => w.write_code(0x190 + (sym - 144), 9),
		256..=279 => w.write_code(sym - 256, 7),
		280..=287 => w.write_code(0xc0 + (sym - 280), 8),
		_ => unreachable!("invalid literal/length symbol {sym}"),
	}
}

/// (length-code, extra-bits, base-length) for lengths 3..=258.
const LENGTH_TABLE: [(u32, u32, u32); 29] = [
	(257, 0, 3),
	(258, 0, 4),
	(259, 0, 5),
	(260, 0, 6),
	(261, 0, 7),
	(262, 0, 8),
	(263, 0, 9),
	(264, 0, 10),
	(265, 1, 11),
	(266, 1, 13),
	(267, 1, 15),
	(268, 1, 17),
	(269, 2, 19),
	(270, 2, 23),
	(271, 2, 27),
	(272, 2, 31),
	(273, 3, 35),
	(274, 3, 43),
	(275, 3, 51),
	(276, 3, 59),
	(277, 4, 67),
	(278, 4, 83),
	(279, 4, 99),
	(280, 4, 115),
	(281, 5, 131),
	(282, 5, 163),
	(283, 5, 195),
	(284, 5, 227),
	(285, 0, 258),
];

/// (distance-code, extra-bits, base-distance) for distances 1..=32768.
const DISTANCE_TABLE: [(u32, u32, u32); 30] = [
	(0, 0, 1),
	(1, 0, 2),
	(2, 0, 3),
	(3, 0, 4),
	(4, 1, 5),
	(5, 1, 7),
	(6, 2, 9),
	(7, 2, 13),
	(8, 3, 17),
	(9, 3, 25),
	(10, 4, 33),
	(11, 4, 49),
	(12, 5, 65),
	(13, 5, 97),
	(14, 6, 129),
	(15, 6, 193),
	(16, 7, 257),
	(17, 7, 385),
	(18, 8, 513),
	(19, 8, 769),
	(20, 9, 1025),
	(21, 9, 1537),
	(22, 10, 2049),
	(23, 10, 3073),
	(24, 11, 4097),
	(25, 11, 6145),
	(26, 12, 8193),
	(27, 12, 12289),
	(28, 13, 16385),
	(29, 13, 24577),
];

fn write_length(w: &mut BitWriter, length: u32) {
	// Find the row whose base is the largest base <= length.
	let mut row = LENGTH_TABLE[0];
	for entry in LENGTH_TABLE.iter() {
		if entry.2 <= length {
			row = *entry;
		} else {
			break;
		}
	}
	let (code, extra_bits, base) = row;
	write_litlen_symbol(w, code);
	if extra_bits > 0 {
		w.write_bits(length - base, extra_bits);
	}
}

fn write_distance(w: &mut BitWriter, distance: u32) {
	let mut row = DISTANCE_TABLE[0];
	for entry in DISTANCE_TABLE.iter() {
		if entry.2 <= distance {
			row = *entry;
		} else {
			break;
		}
	}
	let (code, extra_bits, base) = row;
	// Distance codes are a fixed 5-bit code, MSB-first.
	w.write_code(code, 5);
	if extra_bits > 0 {
		w.write_bits(distance - base, extra_bits);
	}
}

// ---------------------------------------------------------------------------
// LZ77
// ---------------------------------------------------------------------------

const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 258;
const WINDOW_SIZE: usize = 32768;
const HASH_BITS: usize = 15;
const HASH_SIZE: usize = 1 << HASH_BITS;
const MAX_CHAIN: usize = 256;

#[inline]
fn hash3(data: &[u8], pos: usize) -> usize {
	let a = data[pos] as usize;
	let b = data[pos + 1] as usize;
	let c = data[pos + 2] as usize;
	((a << 10) ^ (b << 5) ^ c) & (HASH_SIZE - 1)
}

fn lz77_encode(data: &[u8], w: &mut BitWriter) {
	let n = data.len();
	if n == 0 {
		return;
	}

	let mut head = vec![-1isize; HASH_SIZE];
	let mut prev = vec![-1isize; n];

	let mut i = 0usize;
	while i < n {
		let mut best_len = 0usize;
		let mut best_dist = 0usize;

		if i + MIN_MATCH <= n {
			let h = hash3(data, i);
			let mut candidate = head[h];
			let min_pos = i.saturating_sub(WINDOW_SIZE) as isize;
			let mut chain = 0;

			while candidate >= 0 && candidate >= min_pos && chain < MAX_CHAIN {
				let cand = candidate as usize;
				// Quick reject: only bother if it can beat the current best.
				if best_len == 0 || (cand + best_len < n && data[cand + best_len] == data[i + best_len])
				{
					let mut len = 0usize;
					let max = (n - i).min(MAX_MATCH);
					while len < max && data[cand + len] == data[i + len] {
						len += 1;
					}
					if len > best_len {
						best_len = len;
						best_dist = i - cand;
						if len >= max {
							break;
						}
					}
				}
				candidate = prev[cand];
				chain += 1;
			}
		}

		if best_len >= MIN_MATCH {
			write_length(w, best_len as u32);
			write_distance(w, best_dist as u32);
			// Insert hashes for every covered position so later matches can
			// reference them, then advance past the match.
			let end = i + best_len;
			while i < end {
				if i + MIN_MATCH <= n {
					let h = hash3(data, i);
					prev[i] = head[h];
					head[h] = i as isize;
				}
				i += 1;
			}
		} else {
			write_litlen_symbol(w, data[i] as u32);
			if i + MIN_MATCH <= n {
				let h = hash3(data, i);
				prev[i] = head[h];
				head[h] = i as isize;
			}
			i += 1;
		}
	}
}
