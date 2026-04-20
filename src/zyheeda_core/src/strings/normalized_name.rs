use std::{fmt::Display, ops::Deref, sync::OnceLock};

/// Normalizes names by:
/// - converting to lowercase
/// - removing characters listed in [`REMOVE_CHARS`](Self::REMOVE_CHARS)
#[derive(Debug, Clone)]
pub struct NormalizedName<TName = &'static str>
where
	TName: Deref<Target = str>,
{
	name: TName,
	normalized: OnceLock<String>,
}

impl<TName> NormalizedName<TName>
where
	TName: Deref<Target = str>,
{
	const REMOVE_CHARS: &'static [char] = &['_', ' '];

	pub const fn from_name(name: TName) -> Self {
		Self {
			name,
			normalized: OnceLock::new(),
		}
	}

	pub fn as_str(&self) -> &'_ str {
		self.normalized
			.get_or_init(|| self.name.to_lowercase().replace(Self::REMOVE_CHARS, ""))
	}

	pub fn to_owned(&self) -> String {
		String::from(self.as_str())
	}
}

impl<TName> From<TName> for NormalizedName<TName>
where
	TName: Deref<Target = str>,
{
	fn from(raw: TName) -> Self {
		Self::from_name(raw)
	}
}

impl<TName> From<NormalizedName<TName>> for String
where
	TName: Deref<Target = str>,
{
	fn from(name: NormalizedName<TName>) -> Self {
		name.to_owned()
	}
}

impl<TName> Deref for NormalizedName<TName>
where
	TName: Deref<Target = str>,
{
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl<TName> Display for NormalizedName<TName>
where
	TName: Deref<Target = str>,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(self.as_str(), f)
	}
}

impl<TLeft, TRight> PartialEq<NormalizedName<TRight>> for NormalizedName<TLeft>
where
	TLeft: Deref<Target = str>,
	TRight: Deref<Target = str>,
{
	fn eq(&self, other: &NormalizedName<TRight>) -> bool {
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
	#[test_case("name with spaces", "namewithspaces"; "spaced")]
	fn normalize_name(name: &str, expected: &str) {
		let name = NormalizedName::from(name);

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
		let l = NormalizedName::from_name(l);
		let r = NormalizedName::from_name(r);

		assert_eq!(l, r);
	}
}
