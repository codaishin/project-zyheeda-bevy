pub mod asset_server;

use bevy::asset::{Asset, AssetPath, Handle};

pub trait LoadAsset<TAsset: Asset> {
	fn load_asset<'a, TPath: Into<AssetPath<'a>>>(&self, path: TPath) -> Handle<TAsset>;
}
