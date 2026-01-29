use bevy::asset::AssetPath;
use serde::{Deserialize, Serialize};
use std::{ops::Deref, sync::Arc};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Path(Arc<str>);

impl Path {
	pub fn path(&self) -> &str {
		&self.0
	}
}

impl Deref for Path {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<String> for Path {
	fn from(value: String) -> Self {
		Self(Arc::from(value))
	}
}

impl<'a> From<&'a str> for Path {
	fn from(value: &'a str) -> Self {
		Self(Arc::from(value))
	}
}

impl<'a> From<&'a Path> for AssetPath<'a> {
	fn from(value: &'a Path) -> Self {
		AssetPath::from(value.path().to_owned())
	}
}
