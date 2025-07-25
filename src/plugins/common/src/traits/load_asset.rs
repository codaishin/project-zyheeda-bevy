pub mod asset_server;
pub mod load_context;

use bevy::asset::{Asset, AssetPath, Handle};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Path(String);

impl Path {
	pub fn as_string(&self) -> &String {
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
		Self(value)
	}
}

impl<'a> From<&'a str> for Path {
	fn from(value: &'a str) -> Self {
		Self(value.to_owned())
	}
}

impl From<Path> for AssetPath<'_> {
	fn from(value: Path) -> Self {
		AssetPath::from(value.0)
	}
}

pub trait LoadAsset {
	fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
	where
		TAsset: Asset,
		TPath: Into<AssetPath<'static>> + 'static;
}

pub trait TryLoadAsset {
	fn try_load_asset<TAsset, TPath>(
		&mut self,
		path: TPath,
	) -> Result<Handle<TAsset>, AssetNotFound>
	where
		TAsset: Asset,
		TPath: Into<AssetPath<'static>> + 'static;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AssetNotFound;
