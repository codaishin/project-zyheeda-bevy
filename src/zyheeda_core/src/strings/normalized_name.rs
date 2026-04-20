use std::{fmt::Display, ops::Deref, sync::OnceLock};

/// Normalizes names by:
/// - removing numbered suffixes like `.001`
/// - streamlining different ways of word separation like CamelCase, snake_case, using dots or
///   spaces.
///
/// It is lazy and constructed on first read to allow instantiation in `const` contexts.
#[derive(Debug, Clone)]
pub struct NormalizedNameLazy<TName = &'static str>
where
	TName: Deref<Target = str>,
{
	name: TName,
	normalized: OnceLock<String>,
}

impl<TName> NormalizedNameLazy<TName>
where
	TName: Deref<Target = str>,
{
	const REMOVE_CHARS: &'static [char] = &['_', ' ', '.'];

	pub const fn from_name(name: TName) -> Self {
		Self {
			name,
			normalized: OnceLock::new(),
		}
	}

	pub fn as_str(&self) -> &'_ str {
		self.normalized.get_or_init(|| {
			let name = match self.name.rsplit_once('.') {
				Some(("", suffix)) => suffix,
				Some((name, suffix)) if suffix.chars().all(|c| c.is_ascii_digit()) => name,
				_ => &self.name,
			};

			name.to_lowercase().replace(Self::REMOVE_CHARS, "")
		})
	}

	pub fn to_owned(&self) -> String {
		String::from(self.as_str())
	}
}

impl<TName> From<TName> for NormalizedNameLazy<TName>
where
	TName: Deref<Target = str>,
{
	fn from(raw: TName) -> Self {
		Self::from_name(raw)
	}
}

impl<TName> From<NormalizedNameLazy<TName>> for String
where
	TName: Deref<Target = str>,
{
	fn from(name: NormalizedNameLazy<TName>) -> Self {
		name.to_owned()
	}
}

impl<TName> Deref for NormalizedNameLazy<TName>
where
	TName: Deref<Target = str>,
{
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl<TName> Display for NormalizedNameLazy<TName>
where
	TName: Deref<Target = str>,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(self.as_str(), f)
	}
}

impl<TLeft, TRight> PartialEq<NormalizedNameLazy<TRight>> for NormalizedNameLazy<TLeft>
where
	TLeft: Deref<Target = str>,
	TRight: Deref<Target = str>,
{
	fn eq(&self, other: &NormalizedNameLazy<TRight>) -> bool {
		self.as_str() == other.as_str()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fmt::Debug;
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
		let name = NormalizedNameLazy::from(name);

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

	#[test_case("my_name", "my name"; "when values differ")]
	#[test_case("name", String::from("name"); "when types differ")]
	#[test_case("my_name", String::from("my name"); "when types and values differ")]
	fn partial_eq(l: impl Deref<Target = str> + Debug, r: impl Deref<Target = str> + Debug) {
		let l = NormalizedNameLazy::from_name(l);
		let r = NormalizedNameLazy::from_name(r);

		assert_eq!(l, r);
	}
}
