use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[require(Transform, Visibility)]
pub struct AssetModel {
	pub(crate) scene: Option<Scene>,
}

impl AssetModel {
	pub fn scene<T>(params: T) -> Self
	where
		T: Into<Scene>,
	{
		Self {
			scene: Some(params.into()),
		}
	}

	pub fn none() -> Self {
		Self { scene: None }
	}
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct Scene {
	pub asset_path: String,
	pub id: usize,
}

impl Scene {
	pub const DEFAULT_SCENE_ID: usize = 0;
}

impl From<String> for Scene {
	fn from(asset_path: String) -> Self {
		Self {
			asset_path,
			id: Self::DEFAULT_SCENE_ID,
		}
	}
}

impl From<&'_ String> for Scene {
	fn from(asset_path: &String) -> Self {
		Self {
			asset_path: asset_path.clone(),
			id: Self::DEFAULT_SCENE_ID,
		}
	}
}

impl From<&'_ str> for Scene {
	fn from(asset_path: &str) -> Self {
		Self {
			asset_path: asset_path.into(),
			id: Self::DEFAULT_SCENE_ID,
		}
	}
}

impl<T> From<(T, usize)> for Scene
where
	T: Into<String>,
{
	fn from((asset_path, id): (T, usize)) -> Self {
		Self {
			asset_path: asset_path.into(),
			id,
		}
	}
}
