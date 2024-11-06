use crate::{
	resources::{track::Loaded, AssetFolder},
	traits::is_fully_loaded::IsFullyLoaded,
};
use bevy::prelude::*;

pub(crate) fn is_loaded<TAsset>(
	server: Res<AssetServer>,
	folder: Option<Res<AssetFolder<TAsset>>>,
) -> Loaded
where
	TAsset: Asset,
{
	is_loaded_internal(server, folder)
}

fn is_loaded_internal<TAssetServer, TAsset>(
	server: TAssetServer,
	folder: Option<Res<AssetFolder<TAsset>>>,
) -> Loaded
where
	TAsset: Asset,
	TAssetServer: IsFullyLoaded,
{
	let Some(folder) = folder else {
		return Loaded(false);
	};

	Loaded(server.is_fully_loaded(folder.folder.id()))
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{asset::LoadedFolder, ecs::system::RunSystemOnce};
	use common::{test_tools::utils::new_handle, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Asset, TypePath)]
	struct _Asset;

	#[derive(Resource, NestedMocks)]
	struct _Server {
		mock: Mock_Server,
	}

	#[automock]
	impl IsFullyLoaded for _Server {
		fn is_fully_loaded<TAsset>(&self, id: AssetId<TAsset>) -> bool
		where
			TAsset: Asset,
		{
			self.mock.is_fully_loaded(id)
		}
	}

	fn setup(folder: Option<AssetFolder<_Asset>>) -> App {
		let Some(folder) = folder else {
			return App::new();
		};

		let mut app = App::new();
		app.insert_resource(folder);

		app
	}

	#[test]
	fn fully_loaded() {
		let handle = new_handle();
		let server = _Server::new().with_mock(|mock| {
			mock.expect_is_fully_loaded::<LoadedFolder>()
				.times(1)
				.with(eq(handle.id()))
				.return_const(true);
		});
		let mut app = setup(Some(AssetFolder::new(handle)));

		let loaded = app
			.world_mut()
			.run_system_once_with(server, is_loaded_internal::<In<_Server>, _Asset>);

		assert_eq!(Loaded(true), loaded);
	}

	#[test]
	fn not_fully_loaded() {
		let handle = new_handle();
		let server = _Server::new().with_mock(|mock| {
			mock.expect_is_fully_loaded::<LoadedFolder>()
				.times(1)
				.with(eq(handle.id()))
				.return_const(false);
		});
		let mut app = setup(Some(AssetFolder::new(handle)));

		let loaded = app
			.world_mut()
			.run_system_once_with(server, is_loaded_internal::<In<_Server>, _Asset>);

		assert_eq!(Loaded(false), loaded);
	}

	#[test]
	fn not_fully_loaded_when_asset_folder_resource_does_not_exist() {
		let server = _Server::new().with_mock(|mock| {
			mock.expect_is_fully_loaded::<LoadedFolder>()
				.never()
				.return_const(false);
		});
		let mut app = setup(None);

		let loaded = app
			.world_mut()
			.run_system_once_with(server, is_loaded_internal::<In<_Server>, _Asset>);

		assert_eq!(Loaded(false), loaded);
	}
}
