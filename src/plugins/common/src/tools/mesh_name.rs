use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, ops::Deref, sync::Arc};

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Serialize, Deserialize)]
pub struct MeshName(Arc<str>);

impl From<&str> for MeshName {
	fn from(value: &str) -> Self {
		Self(Arc::from(value))
	}
}

impl From<&Name> for MeshName {
	fn from(value: &Name) -> Self {
		Self(Arc::from(value.as_str()))
	}
}

impl From<MeshName> for Name {
	fn from(bone: MeshName) -> Self {
		Name::from(&*bone)
	}
}

impl PartialEq<Name> for MeshName {
	fn eq(&self, other: &Name) -> bool {
		&*self.0 == other.as_str()
	}
}

impl PartialEq<MeshName> for Name {
	fn eq(&self, other: &MeshName) -> bool {
		self.as_str() == &*other.0
	}
}

impl Deref for MeshName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Borrow<str> for MeshName {
	fn borrow(&self) -> &str {
		&self.0
	}
}
