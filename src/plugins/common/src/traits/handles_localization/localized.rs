use bevy::prelude::*;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Localized(pub String);

impl Localized {
	pub fn from_string<T>(value: T) -> Self
	where
		T: Into<String>,
	{
		Self(value.into())
	}
}

impl From<&str> for Localized {
	fn from(value: &str) -> Self {
		Self::from_string(value)
	}
}

impl From<String> for Localized {
	fn from(value: String) -> Self {
		Self::from_string(value)
	}
}

impl From<Localized> for String {
	fn from(Localized(string): Localized) -> Self {
		string
	}
}

impl From<Localized> for Text {
	fn from(Localized(string): Localized) -> Self {
		Text(string)
	}
}

impl From<Localized> for Name {
	fn from(Localized(string): Localized) -> Self {
		Name::from(string)
	}
}
