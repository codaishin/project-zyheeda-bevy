pub mod asset_server;

use bevy::asset::{AssetPath, UntypedAssetId};

pub trait GetAssetPath {
	fn get_asset_path<T: Into<UntypedAssetId>>(&self, id: T) -> Option<AssetPath<'_>>;
}
