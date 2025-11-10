use super::LoadFolderAssets;
use bevy::asset::{AssetPath, AssetServer, Handle, LoadedFolder};

impl LoadFolderAssets for AssetServer {
	fn load_folder_assets<'a, TPath>(&self, path: TPath) -> Handle<LoadedFolder>
	where
		TPath: Into<AssetPath<'a>>,
	{
		self.load_folder(path)
	}
}
