use super::LoadAsset;
use crate::traits::load_asset::{AssetNotFound, TryLoadAsset};
use bevy::{asset::AssetPath, prelude::*};
use std::path::PathBuf;

impl LoadAsset for AssetServer {
	fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
	where
		TAsset: Asset,
		TPath: Into<AssetPath<'static>>,
	{
		self.load(path)
	}
}

const ASSETS_ROOT: &str = "assets";

impl TryLoadAsset for AssetServer {
	fn try_load_asset<TAsset, TPath>(
		&mut self,
		path: TPath,
	) -> Result<Handle<TAsset>, AssetNotFound>
	where
		TAsset: Asset,
		TPath: Into<AssetPath<'static>> + 'static,
	{
		let path = path.into();
		let asset_path = PathBuf::from(ASSETS_ROOT).join(path.path());
		if !asset_path.exists() {
			return Err(AssetNotFound);
		}

		Ok(self.load(path))
	}
}
