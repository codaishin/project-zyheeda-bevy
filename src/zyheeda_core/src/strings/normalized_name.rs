use std::{cell::OnceCell, fmt::Display, ops::Deref};

/// Normalizes names by:
/// - converting to lowercase
/// - removing characters listed in [`REMOVE_CHARS`](Self::REMOVE_CHARS)
#[derive(Debug, Clone)]
pub struct NormalizedName<'a> {
	raw: &'a str,
	normalized: OnceCell<String>,
}

impl<'a> NormalizedName<'a> {
	const REMOVE_CHARS: &'static [char] = &['_', ' '];

	pub const fn from_raw(raw: &'a str) -> Self {
		Self {
			raw,
			normalized: OnceCell::new(),
		}
	}

	pub fn as_str(&self) -> &'_ str {
		self.normalized
			.get_or_init(|| self.raw.to_lowercase().replace(Self::REMOVE_CHARS, ""))
	}

	pub fn to_owned(&self) -> String {
		String::from(self.as_str())
	}
}

impl<'a> From<&'a str> for NormalizedName<'a> {
	fn from(raw: &'a str) -> Self {
		Self::from_raw(raw)
	}
}

impl<'a> From<NormalizedName<'a>> for String {
	fn from(name: NormalizedName<'a>) -> Self {
		name.to_owned()
	}
}

impl<'a> Deref for NormalizedName<'a> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl<'a> Display for NormalizedName<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(self.as_str(), f)
	}
}

impl<'a> PartialEq for NormalizedName<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.as_str() == other.as_str()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test_case("normalized", "normalized"; "unchanged")]
	#[test_case("CamelCase", "camelcase"; "camel case")]
	#[test_case("snake_case", "snakecase"; "snake case")]
	#[test_case("name with spaces", "namewithspaces"; "spaced")]
	fn normalize_name(raw: &str, expected: &str) {
		let name = NormalizedName::from(raw);

		assert_eq!(
			(
				expected,
				expected,
				expected.to_owned(),
				expected.to_owned(),
				expected.to_owned()
			),
			(
				name.clone().as_str(),
				name.clone().deref(),
				name.to_string(),
				name.to_owned(),
				String::from(name),
			)
		);
	}

	#[test]
	fn partial_eq_of_different_raw_values() {
		let a = NormalizedName::from_raw("my_name");
		let b = NormalizedName::from_raw("my name");

		assert_eq!(a, b);
	}
}
