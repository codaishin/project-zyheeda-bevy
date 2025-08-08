use super::GetAssetPath;
use bevy::asset::{AssetPath, AssetServer, UntypedAssetId};

impl GetAssetPath for AssetServer {
	fn get_asset_path<T: Into<UntypedAssetId>>(&self, id: T) -> Option<AssetPath<'_>> {
		self.get_path(id)
	}
}
