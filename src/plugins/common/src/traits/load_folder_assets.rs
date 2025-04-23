pub mod asset_server;

use bevy::asset::{AssetPath, Handle, LoadedFolder};

pub trait LoadFolderAssets {
	fn load_folder_assets<TPath>(&self, path: TPath) -> Handle<LoadedFolder>
	where
		TPath: Into<AssetPath<'static>> + 'static;
}
