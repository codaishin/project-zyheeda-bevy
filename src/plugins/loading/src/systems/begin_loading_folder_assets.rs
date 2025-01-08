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
	let folder = asset_server.load_folder_assets(TAsset::asset_folder_path());
	commands.insert_resource(AssetFolder::<TAsset>::new(folder));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle, LoadedFolder},
		prelude::Resource,
		reflect::TypePath,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{load_asset::Path, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use uuid::Uuid;

	#[derive(Resource, NestedMocks)]
	struct _Server {
		mock: Mock_Server,
	}

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Asset;

	impl AssetFolderPath for _Asset {
		fn asset_folder_path() -> Path {
			Path::from("my/asset/folder/path")
		}
	}

	#[automock]
	impl LoadFolderAssets for _Server {
		fn load_folder_assets(&self, path: Path) -> Handle<LoadedFolder> {
			self.mock.load_folder_assets(path)
		}
	}

	fn setup(server: _Server) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(server);
		app.add_systems(Update, begin_loading_folder_assets::<_Asset, _Server>);

		app
	}

	#[test]
	fn store_folder_handle() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(_Server::new().with_mock(|mock| {
			mock.expect_load_folder_assets()
				.return_const(handle.clone());
		}));

		app.update();

		let asset_folder = app.world().resource::<AssetFolder<_Asset>>();

		assert_eq!(&AssetFolder::<_Asset>::new(handle), asset_folder);
	}

	#[test]
	fn call_load_folder_assets_with_asset_path() {
		let mut app = setup(_Server::new().with_mock(|mock| {
			mock.expect_load_folder_assets()
				.times(1)
				.with(eq(_Asset::asset_folder_path()))
				.return_const(Handle::default());
		}));

		app.update();
	}
}
