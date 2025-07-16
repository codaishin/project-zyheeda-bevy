use crate::traits::pixels::Bytes;
use bevy::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct ParsedColor(Option<Color>);

impl ParsedColor {
	pub(crate) fn parse(bytes: Bytes) -> Self {
		match bytes {
			[r, g, b, a] => Self(Some(Color::srgba_u8(*r, *g, *b, *a))),
			[r, g, b] => Self(Some(Color::srgb_u8(*r, *g, *b))),
			_ => Self(None),
		}
	}

	pub(crate) fn color(&self) -> Option<&Color> {
		self.0.as_ref()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test]
	fn parse_4_bytes() {
		assert_eq!(
			ParsedColor(Some(Color::srgba_u8(1, 2, 3, 4))),
			ParsedColor::parse(&[1, 2, 3, 4]),
		);
	}

	#[test]
	fn parse_3_bytes() {
		assert_eq!(
			ParsedColor(Some(Color::srgb_u8(1, 2, 3))),
			ParsedColor::parse(&[1, 2, 3]),
		);
	}

	#[test_case(&[1]; "one byte")]
	#[test_case(&[1, 1]; "two byte")]
	#[test_case(&[1, 1, 1, 1, 1]; "five byte")]
	fn parse_none(bytes: Bytes) {
		assert_eq!(ParsedColor(None), ParsedColor::parse(bytes),);
	}
}
