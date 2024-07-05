pub mod asset_server;

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

pub trait LoadAsset<TAsset: Asset> {
	fn load_asset(&mut self, path: Path) -> Handle<TAsset>;
}
