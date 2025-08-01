use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[require(Transform, Visibility)]
pub struct AssetModel {
	pub(crate) flip_horizontally: Option<Name>,
	pub(crate) model: Model,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) enum Model {
	#[default]
	None,
	Path(String),
}

impl<T> From<T> for AssetModel
where
	T: Into<String>,
{
	fn from(path: T) -> Self {
		Self::path(path)
	}
}

impl AssetModel {
	pub fn path<T>(path: T) -> Self
	where
		T: Into<String>,
	{
		Self {
			flip_horizontally: None,
			model: Model::Path(path.into()),
		}
	}

	pub fn none() -> Self {
		Self {
			flip_horizontally: None,
			model: Model::None,
		}
	}

	pub fn flipped_on<T>(mut self, name: T) -> Self
	where
		T: Into<Name>,
	{
		self.flip_horizontally = Some(name.into());
		self
	}
}
