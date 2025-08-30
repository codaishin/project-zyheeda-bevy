use bevy::prelude::*;
use std::{ops::Deref, sync::Arc};

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Localized(pub Arc<str>);

impl Deref for Localized {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.as_ref()
	}
}

impl<T> From<T> for Localized
where
	T: Into<String>,
{
	fn from(value: T) -> Self {
		Self(Arc::from(value.into()))
	}
}

impl From<Localized> for Text {
	fn from(Localized(string): Localized) -> Self {
		Text((*string).to_owned())
	}
}

impl From<&Localized> for Text {
	fn from(Localized(string): &Localized) -> Self {
		Text((**string).to_owned())
	}
}

impl From<Localized> for Name {
	fn from(Localized(string): Localized) -> Self {
		Name::from(&*string)
	}
}

impl From<&Localized> for Name {
	fn from(Localized(string): &Localized) -> Self {
		Name::from(&**string)
	}
}
