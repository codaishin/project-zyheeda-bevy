use super::GetHandelFromPath;
use crate::traits::load_asset::Path;
use bevy::asset::{Asset, Handle, LoadedFolder, UntypedHandle};

impl<T: Asset> GetHandelFromPath<T> for LoadedFolder {
	fn handle_from_path(&self, path: &Path) -> Option<Handle<T>> {
		self.handles
			.iter()
			.filter_map(to_typed)
			.find(asset_path_ends_with(path))
	}
}

fn asset_path_ends_with<T: Asset>(path: &Path) -> impl FnMut(&Handle<T>) -> bool + '_ {
	move |handle| match handle.path() {
		None => false,
		Some(asset_path) => asset_path.path().ends_with(path.as_string()),
	}
}

fn to_typed<T: Asset>(handle: &UntypedHandle) -> Option<Handle<T>> {
	handle.clone().try_typed::<T>().ok()
}
