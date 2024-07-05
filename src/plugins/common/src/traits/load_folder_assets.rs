pub mod asset_server;

use super::load_asset::Path;
use bevy::asset::{Handle, LoadedFolder};

pub trait LoadFolderAssets {
	fn load_folder_assets(&self, path: Path) -> Handle<LoadedFolder>;
}
