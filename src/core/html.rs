// TODO: analyse
use std::borrow::Cow;
use std::fmt::Write;

static ESCAPE_TABLE: [Option<&'static str>; 128] = {
	let mut table: [Option<&'static str>; 128] = [None; 128];
	table[b'&' as usize] = Some("&amp;");
	table[b'<' as usize] = Some("&lt;");
	table[b'>' as usize] = Some("&gt;");
	table[b'"' as usize] = Some("&quot;");
	table[b'\'' as usize] = Some("&#x27;");
	table
};

static ESCAPE_TABLE_MINIMAL: [Option<&'static str>; 128] = {
	let mut table: [Option<&'static str>; 128] = [None; 128];
	table[b'&' as usize] = Some("&amp;");
	table[b'<' as usize] = Some("&lt;");
	table[b'>' as usize] = Some("&gt;");
	table
};

static ESCAPE_TABLE_ATTR: [Option<&'static str>; 128] = {
	let mut table: [Option<&'static str>; 128] = [None; 128];
	table[b'&' as usize] = Some("&amp;");
	table[b'<' as usize] = Some("&lt;");
	table[b'>' as usize] = Some("&gt;");
	table[b'"' as usize] = Some("&quot;");
	table[b'\'' as usize] = Some("&#x27;");
	table[b'/' as usize] = Some("&#x2F;");
	table[b'`' as usize] = Some("&#x60;");
	table[b'=' as usize] = Some("&#x3D;");
	table
};

const NEEDS_ESCAPE: u128 = {
	let mut bits: u128 = 0;
	bits |= 1u128 << (b'&' as u32);
	bits |= 1u128 << (b'<' as u32);
	bits |= 1u128 << (b'>' as u32);
	bits |= 1u128 << (b'"' as u32);
	bits |= 1u128 << (b'\'' as u32);
	bits
};

const NEEDS_ESCAPE_MINIMAL: u128 = {
	let mut bits: u128 = 0;
	bits |= 1u128 << (b'&' as u32);
	bits |= 1u128 << (b'<' as u32);
	bits |= 1u128 << (b'>' as u32);
	bits
};

#[inline(always)]
fn needs_escape(byte: u8) -> bool {
	byte < 128 && (NEEDS_ESCAPE & (1u128 << byte)) != 0
}

#[inline(always)]
fn needs_escape_minimal(byte: u8) -> bool {
	byte < 128 && (NEEDS_ESCAPE_MINIMAL & (1u128 << byte)) != 0
}

#[inline]
fn find_escape_index(bytes: &[u8]) -> Option<usize> {
	for (i, &byte) in bytes.iter().enumerate() {
		if needs_escape(byte) {
			return Some(i);
		}
	}
	None
}

#[inline]
fn find_escape_index_minimal(bytes: &[u8]) -> Option<usize> {
	for (i, &byte) in bytes.iter().enumerate() {
		if needs_escape_minimal(byte) {
			return Some(i);
		}
	}
	None
}

#[inline]
pub fn escape(input: &str) -> Cow<'_, str> {
	let bytes = input.as_bytes();

	match find_escape_index(bytes) {
		None => Cow::Borrowed(input),
		Some(first) => {
			let mut output = String::with_capacity(input.len() + input.len() / 5);
			output.push_str(&input[..first]);

			escape_to_buf(&input[first..], &mut output, &ESCAPE_TABLE);
			Cow::Owned(output)
		}
	}
}

#[inline]
pub fn escape_minimal(input: &str) -> Cow<'_, str> {
	let bytes = input.as_bytes();

	match find_escape_index_minimal(bytes) {
		None => Cow::Borrowed(input),
		Some(first) => {
			let mut output = String::with_capacity(input.len() + input.len() / 5);
			output.push_str(&input[..first]);
			escape_to_buf(&input[first..], &mut output, &ESCAPE_TABLE_MINIMAL);
			Cow::Owned(output)
		}
	}
}

#[inline]
pub fn escape_attribute(input: &str) -> Cow<'_, str> {
	let bytes = input.as_bytes();
	let needs = bytes
		.iter()
		.any(|&b| b < 128 && matches!(b, b'&' | b'<' | b'>' | b'"' | b'\'' | b'/' | b'`' | b'='));

	if !needs {
		return Cow::Borrowed(input);
	}

	let mut output = String::with_capacity(input.len() + input.len() / 4);
	escape_to_buf(input, &mut output, &ESCAPE_TABLE_ATTR);
	Cow::Owned(output)
}

#[inline]
pub fn escape_to_string(input: &str, output: &mut String) {
	escape_to_buf(input, output, &ESCAPE_TABLE);
}

pub fn escape_to_writer<W: Write>(input: &str, writer: &mut W) -> std::fmt::Result {
	for byte in input.bytes() {
		if byte < 128 {
			if let Some(escaped) = ESCAPE_TABLE[byte as usize] {
				writer.write_str(escaped)?;
				continue;
			}
		}
		writer.write_char(byte as char)?;
	}
	Ok(())
}

#[inline]
fn escape_to_buf(input: &str, output: &mut String, table: &[Option<&'static str>; 128]) {
	for byte in input.bytes() {
		if byte < 128 {
			if let Some(escaped) = table[byte as usize] {
				output.push_str(escaped);
				continue;
			}
		}
		// SAFETY: We're iterating byte-by-byte through a valid UTF-8 string.
		// Non-ASCII bytes are part of multi-byte sequences and are pushed as-is.
		output.push(byte as char);
	}
}

pub fn unescape(input: &str) -> Cow<'_, str> {
	if !input.contains('&') {
		return Cow::Borrowed(input);
	}

	let mut output = String::with_capacity(input.len());
	let mut chars = input.chars().peekable();

	while let Some(ch) = chars.next() {
		if ch == '&' {
			let mut entity = String::with_capacity(8);
			entity.push('&');

			while let Some(&next) = chars.peek() {
				if next == ';' || entity.len() > 10 {
					break;
				}
				entity.push(chars.next().unwrap());
			}

			if chars.peek() == Some(&';') {
				entity.push(chars.next().unwrap());
			}

			match entity.as_str() {
				"&amp;" => output.push('&'),
				"&lt;" => output.push('<'),
				"&gt;" => output.push('>'),
				"&quot;" => output.push('"'),
				"&#x27;" | "&#39;" | "&apos;" => output.push('\''),
				"&#x2F;" | "&#47;" => output.push('/'),
				"&#x60;" | "&#96;" => output.push('`'),
				"&#x3D;" | "&#61;" => output.push('='),
				"&nbsp;" => output.push('\u{00A0}'),
				_ => {
					if let Some(decoded) = decode_numeric_entity(&entity) {
						output.push(decoded);
					} else {
						output.push_str(&entity);
					}
				}
			}
		} else {
			output.push(ch);
		}
	}

	Cow::Owned(output)
}

fn decode_numeric_entity(entity: &str) -> Option<char> {
	if !entity.starts_with("&#") || !entity.ends_with(';') {
		return None;
	}

	let inner = &entity[2..entity.len() - 1];
	let codepoint = if let Some(hex) = inner.strip_prefix('x').or_else(|| inner.strip_prefix('X')) {
		u32::from_str_radix(hex, 16).ok()?
	} else {
		inner.parse::<u32>().ok()?
	};

	char::from_u32(codepoint)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_escape_basic() {
		assert_eq!(escape("hello"), "hello");
		assert_eq!(escape("<>"), "&lt;&gt;");
		assert_eq!(escape("&"), "&amp;");
		assert_eq!(escape("\""), "&quot;");
		assert_eq!(escape("'"), "&#x27;");
	}

	#[test]
	fn test_escape_mixed() {
		assert_eq!(
			escape("<script>alert('xss')</script>"),
			"&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
		);
	}

	#[test]
	fn test_escape_no_allocation() {
		let input = "no special chars here";
		let result = escape(input);
		assert!(matches!(result, Cow::Borrowed(_)));
	}

	#[test]
	fn test_escape_minimal() {
		assert_eq!(escape_minimal("<div>"), "&lt;div&gt;");
		assert_eq!(escape_minimal("\"'"), "\"'");
	}

	#[test]
	fn test_escape_attribute() {
		let result = escape_attribute("onclick='alert(1)'");
		assert!(result.contains("&#x27;"));
	}

	#[test]
	fn test_escape_to_string() {
		let mut output = String::new();
		escape_to_string("<test>", &mut output);
		assert_eq!(output, "&lt;test&gt;");
	}

	#[test]
	fn test_unescape() {
		assert_eq!(unescape("&lt;div&gt;"), "<div>");
		assert_eq!(unescape("&amp;&quot;"), "&\"");
		assert_eq!(unescape("&#x27;"), "'");
		assert_eq!(unescape("&#39;"), "'");
	}

	#[test]
	fn test_unescape_numeric() {
		assert_eq!(unescape("&#65;"), "A");
		assert_eq!(unescape("&#x41;"), "A");
	}

	#[test]
	fn test_unescape_no_allocation() {
		let input = "no entities here";
		let result = unescape(input);
		assert!(matches!(result, Cow::Borrowed(_)));
	}

	#[test]
	fn test_roundtrip() {
		let original = "<script>alert('test' & \"xss\")</script>";
		let escaped = escape(original);
		let unescaped = unescape(&escaped);
		assert_eq!(unescaped, original);
	}

	#[test]
	fn test_unicode_passthrough() {
		let input = "Hello 世界";
		assert_eq!(escape(input), "Hello 世界");
	}

	#[test]
	fn test_needs_escape_bitset() {
		assert!(needs_escape(b'&'));
		assert!(needs_escape(b'<'));
		assert!(needs_escape(b'>'));
		assert!(needs_escape(b'"'));
		assert!(needs_escape(b'\''));
		assert!(!needs_escape(b'a'));
		assert!(!needs_escape(b' '));
	}
}
