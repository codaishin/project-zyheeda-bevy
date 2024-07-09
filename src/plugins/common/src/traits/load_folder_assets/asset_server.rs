use super::LoadFolderAssets;
use crate::traits::load_asset::Path;
use bevy::asset::{AssetServer, Handle, LoadedFolder};

impl LoadFolderAssets for AssetServer {
	fn load_folder_assets(&self, path: Path) -> Handle<LoadedFolder> {
		self.load_folder(path)
	}
}
