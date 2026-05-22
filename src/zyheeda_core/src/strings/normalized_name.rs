use std::{fmt::Display, ops::Deref};

/// Normalizes names by:
/// - removing numbered suffixes like `.001`
/// - streamlining different ways of word separation like CamelCase, snake_case, using dots or
///   spaces.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct NormalizedName(String);

impl NormalizedName {
	const REMOVE_CHARS: &'static [char] = &['_', ' ', '.'];
}

impl<TName> From<TName> for NormalizedName
where
	TName: Deref<Target = str>,
{
	fn from(name: TName) -> Self {
		let name = match name.rsplit_once('.') {
			Some(("", suffix)) => suffix,
			Some((name, suffix)) if suffix.chars().all(|c| c.is_ascii_digit()) => name,
			_ => &name,
		};

		Self(name.to_lowercase().replace(Self::REMOVE_CHARS, ""))
	}
}

impl Display for NormalizedName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test_case("normalized", "normalized"; "unchanged")]
	#[test_case("CamelCase", "camelcase"; "camel case")]
	#[test_case("snake_case", "snakecase"; "snake case")]
	#[test_case("dotted.name", "dottedname"; "with dots")]
	#[test_case("name with spaces", "namewithspaces"; "with spaces")]
	#[test_case("normalized.001", "normalized"; "with number suffix leading with a dot")]
	#[test_case("number_42_is_best", "number42isbest"; "with non suffix digits")]
	#[test_case("normalized001", "normalized001"; "with digits at the end without leading dot")]
	#[test_case("001", "001"; "with only digits")]
	#[test_case(".001", "001"; "with only suffix leading with a dot")]
	fn normalize_name(name: &str, expected: &str) {
		let name = NormalizedName::from(name);

		assert_eq!(NormalizedName(expected.to_owned()), name);
	}
}
