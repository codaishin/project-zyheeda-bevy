pub mod asset_server;
pub mod load_context;

use bevy::asset::{Asset, AssetPath, Handle};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Path(String);

impl Path {
	pub fn as_string(&self) -> &String {
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

impl<'a> From<Path> for AssetPath<'a> {
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
