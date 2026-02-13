use crate::core::error::MurError;

pub struct MurCodec;

impl MurCodec {
	pub fn url_encode(s: &str) -> String {
		urlencoding::encode(s).into_owned()
	}

	pub fn url_decode(s: &str) -> Result<String, MurError> {
		urlencoding::decode(s)
			.map(|s| s.into_owned())
			.map_err(|e| MurError::BadRequest(format!("Invalid URL encoding: {}", e)))
	}

	pub fn base64_encode(data: &[u8]) -> String {
		const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

		let mut result = String::new();
		let mut i = 0;

		while i < data.len() {
			let b0 = data[i] as usize;
			let b1 = if i + 1 < data.len() {
				data[i + 1] as usize
			} else {
				0
			};
			let b2 = if i + 2 < data.len() {
				data[i + 2] as usize
			} else {
				0
			};

			result.push(CHARS[b0 >> 2] as char);
			result.push(CHARS[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

			if i + 1 < data.len() {
				result.push(CHARS[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
			} else {
				result.push('=');
			}

			if i + 2 < data.len() {
				result.push(CHARS[b2 & 0x3f] as char);
			} else {
				result.push('=');
			}

			i += 3;
		}

		result
	}

	pub fn base64_decode(s: &str) -> Result<Vec<u8>, MurError> {
		const DECODE: [i8; 128] = [
			-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
			-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62,
			-1, -1, -1, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1, -1, 0,
			1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
			25, -1, -1, -1, -1, -1, -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
			41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1,
		];

		let decode_error: MurError = MurError::BadRequest(String::from("Invalid base64"));
		let s = s.trim_end_matches('=');
		let mut result = Vec::new();
		let mut buf = 0u32;
		let mut bits = 0;

		for c in s.bytes() {
			if c >= 128 {
				return Err(decode_error);
			}
			let val = DECODE[c as usize];
			if val < 0 {
				return Err(decode_error);
			}

			buf = (buf << 6) | (val as u32);
			bits += 6;

			if bits >= 8 {
				bits -= 8;
				result.push((buf >> bits) as u8);
				buf &= (1 << bits) - 1;
			}
		}

		Ok(result)
	}
}
