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
	Scene(String),
}

impl AssetModel {
	pub fn scene<T>(path: T) -> Self
	where
		T: Into<String>,
	{
		Self {
			model: Model::Scene(path.into()),
		}
	}

	pub fn none() -> Self {
		Self { model: Model::None }
	}
}
