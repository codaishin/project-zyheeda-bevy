use bevy::prelude::*;
use macros::serde_model;
use std::{borrow::Borrow, ops::Deref, sync::Arc};

#[serde_model]
#[derive(Debug, PartialEq, Eq, Hash, Default, Clone)]
pub struct BoneName(Arc<str>);

impl From<&str> for BoneName {
	fn from(value: &str) -> Self {
		Self(Arc::from(value))
	}
}

impl From<&Name> for BoneName {
	fn from(value: &Name) -> Self {
		Self(Arc::from(value.as_str()))
	}
}

impl From<BoneName> for Name {
	fn from(bone: BoneName) -> Self {
		Name::from(&*bone)
	}
}

impl PartialEq<Name> for BoneName {
	fn eq(&self, other: &Name) -> bool {
		&*self.0 == other.as_str()
	}
}

impl PartialEq<BoneName> for Name {
	fn eq(&self, other: &BoneName) -> bool {
		self.as_str() == &*other.0
	}
}

impl Deref for BoneName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Borrow<str> for BoneName {
	fn borrow(&self) -> &str {
		&self.0
	}
}
