use std::fmt::Display;

#[macro_export]
macro_rules! u8_array_from_hex {
	($hex:expr) => {{
		const A: [u8; $hex.len() / 2] = match $crate::math::parse::u8_array_try_from_hex($hex) {
			Ok(v) => v,
			Err($crate::math::parse::U8FromHexError::InvalidCharacter) => {
				panic!("Invalid character. Allowed: [0-9], [A-F], [a-f]")
			}
			Err($crate::math::parse::U8FromHexError::OddHexLen) => {
				panic!("Hex character count must not be odd")
			}
			Err($crate::math::parse::U8FromHexError::HexLenMismatch) => {
				panic!("Hex character count is not double of array size")
			}
		};

		A
	}};
}

pub use u8_array_from_hex;

pub const fn u8_array_try_from_hex<const N: usize>(hex: &str) -> Result<[u8; N], U8FromHexError> {
	if !hex.len().is_multiple_of(2) {
		return Err(U8FromHexError::OddHexLen);
	}

	if hex.len() / 2 != N {
		return Err(U8FromHexError::HexLenMismatch);
	}

	let bytes = hex.as_bytes();
	let mut array = [0; N];
	let mut byte_index = 0;

	while byte_index < bytes.len() {
		let array_index = byte_index / 2;

		let v = match bytes[byte_index] {
			v if v >= ascii::ZERO && v <= ascii::NINE => v - ascii::ZERO,
			v if v >= ascii::A_UPPER && v <= ascii::F_UPPER => 10 + v - ascii::A_UPPER,
			v if v >= ascii::A_LOWER && v <= ascii::F_LOWER => 10 + v - ascii::A_LOWER,
			_ => return Err(U8FromHexError::InvalidCharacter),
		};

		let factor_index = byte_index.div_ceil(2) - array_index;
		array[array_index] = array[array_index] * FACTORS[factor_index] + v;

		byte_index += 1;
	}

	Ok(array)
}

#[derive(Debug, PartialEq)]
pub enum U8FromHexError {
	InvalidCharacter,
	OddHexLen,
	HexLenMismatch,
}

impl U8FromHexError {
	pub const INVALID_CHARACTER: &str = "Invalid character. Allowed: [0-9], [A-F], [a-f]";
	pub const ODD_HEX_LEN: &str = "Hex character count must not be odd";
	pub const HEX_LEN_MISMATCH: &str = "Hex character count is not double of array size";
}

impl Display for U8FromHexError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			U8FromHexError::InvalidCharacter => write!(f, "{}", Self::INVALID_CHARACTER),
			U8FromHexError::OddHexLen => write!(f, "{}", Self::ODD_HEX_LEN),
			U8FromHexError::HexLenMismatch => write!(f, "{}", Self::HEX_LEN_MISMATCH),
		}
	}
}

const FACTORS: [u8; 2] = [1, 16];

mod ascii {
	pub(super) const ZERO: u8 = 48;
	pub(super) const NINE: u8 = 57;
	pub(super) const A_UPPER: u8 = 65;
	pub(super) const F_UPPER: u8 = 70;
	pub(super) const A_LOWER: u8 = 97;
	pub(super) const F_LOWER: u8 = 102;
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use test_case::test_case;

	#[test_case("00", [0]; "0")]
	#[test_case("04", [4]; "4")]
	#[test_case("09", [9]; "9")]
	#[test_case("0a", [10]; "lower case 10")]
	#[test_case("0c", [12]; "lower case 12")]
	#[test_case("0f", [15]; "lower case 15")]
	#[test_case("0A", [10]; "capitalized 10")]
	#[test_case("0C", [12]; "capitalized 12")]
	#[test_case("0F", [15]; "capitalized 15")]
	fn single(hex: &str, exp: [u8; 1]) {
		assert_eq!(Ok(exp), u8_array_try_from_hex(hex));
	}

	#[test_case("0g"; "lower case")]
	#[test_case("0G"; "capitalized")]
	fn invalid_character(hex: &str) {
		assert_eq!(
			Err(U8FromHexError::InvalidCharacter),
			u8_array_try_from_hex::<1>(hex)
		);
	}

	#[test]
	fn double() {
		assert_eq!(
			Ok([u8::from_str_radix("5c", 16).unwrap()]),
			u8_array_try_from_hex("5c")
		);
	}

	#[test_case("aabb"; "by one")]
	#[test_case("aabbcc"; "by two")]
	fn array_too_small(hex: &str) {
		assert_eq!(
			Err(U8FromHexError::HexLenMismatch),
			u8_array_try_from_hex::<1>(hex)
		);
	}

	#[test]
	fn hex_number_odd() {
		assert_eq!(
			Err(U8FromHexError::OddHexLen),
			u8_array_try_from_hex::<2>("aab")
		);
	}

	#[test]
	fn long() {
		assert_eq!(
			Ok([
				u8::from_str_radix("5c", 16).unwrap(),
				u8::from_str_radix("ff", 16).unwrap(),
				u8::from_str_radix("12", 16).unwrap()
			]),
			u8_array_try_from_hex("5cff12")
		);
	}

	#[test]
	fn via_macro() {
		assert_eq!(
			[
				u8::from_str_radix("5c", 16).unwrap(),
				u8::from_str_radix("ff", 16).unwrap(),
				u8::from_str_radix("12", 16).unwrap()
			],
			u8_array_from_hex!("5cff12")
		);
	}
}
