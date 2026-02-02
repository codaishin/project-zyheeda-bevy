use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[require(Transform, Visibility)]
pub struct AssetModel {
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
			model: Model::Path(path.into()),
		}
	}

	pub fn none() -> Self {
		Self { model: Model::None }
	}
}
