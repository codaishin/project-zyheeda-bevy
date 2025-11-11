pub mod loaded_folder;

use crate::tools::path::Path;
use bevy::asset::{Asset, Handle};

pub trait GetHandelFromPath<T: Asset> {
	fn handle_from_path(&self, path: &Path) -> Option<Handle<T>>;
}
