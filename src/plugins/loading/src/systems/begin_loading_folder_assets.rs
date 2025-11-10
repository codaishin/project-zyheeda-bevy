use crate::resources::asset_folder::AssetFolder;
use bevy::prelude::*;
use common::traits::{
	handles_custom_assets::AssetFolderPath,
	load_folder_assets::LoadFolderAssets,
};

pub(crate) fn begin_loading_folder_assets<
	TAsset: Asset + AssetFolderPath,
	TAssetServer: LoadFolderAssets + Resource,
>(
	mut commands: Commands,
	asset_server: Res<TAssetServer>,
) {
	let folder = asset_server.load_folder_assets(&TAsset::asset_folder_path());
	commands.insert_resource(AssetFolder::<TAsset>::new(folder));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		reflect::TypePath,
	};
	use common::traits::{load_asset::Path, load_folder_assets::mock::MockFolderAssetServer};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Asset;

	impl _Asset {
		const FOLDER_PATH: &str = "my/asset/folder/path";
	}

	impl AssetFolderPath for _Asset {
		fn asset_folder_path() -> Path {
			Path::from(Self::FOLDER_PATH)
		}
	}

	fn setup(server: MockFolderAssetServer) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(server);
		app.add_systems(
			Update,
			begin_loading_folder_assets::<_Asset, MockFolderAssetServer>,
		);

		app
	}

	#[test]
	fn store_folder_handle() {
		let handle = new_handle();
		let mut app = setup(
			MockFolderAssetServer::default()
				.path(_Asset::FOLDER_PATH)
				.returns(handle.clone()),
		);

		app.update();

		let asset_folder = app.world().resource::<AssetFolder<_Asset>>();

		assert_eq!(&AssetFolder::<_Asset>::new(handle), asset_folder);
	}
}
