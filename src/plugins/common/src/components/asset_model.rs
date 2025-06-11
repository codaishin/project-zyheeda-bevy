use crate::components::flip::FlipHorizontally;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[require(Transform, Visibility)]
pub enum AssetModel {
	#[default]
	None,
	Path(String),
}

impl From<&str> for AssetModel {
	fn from(path: &str) -> Self {
		Self::Path(path.to_owned())
	}
}

impl AssetModel {
	pub fn path(path: &str) -> AssetModel {
		AssetModel::Path(path.to_owned())
	}

	pub fn flip_on(self, name: Name) -> (Self, FlipHorizontally) {
		(self, FlipHorizontally::with(name))
	}
}
