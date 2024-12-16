use super::{GetOrLoadAsset, GetOrLoadAssetFactory, Storage};
use crate::{
	tools::Factory,
	traits::load_asset::{LoadAsset, Path},
};
use bevy::prelude::*;

pub struct LoadAssetCache;

impl<TAssets, TAsset, TStorage> GetOrLoadAssetFactory<TAssets, TAsset, TStorage>
	for Factory<LoadAssetCache>
where
	TAssets: Resource + LoadAsset,
	TAsset: Asset,
	TStorage: Resource + Storage<Path, Handle<TAsset>>,
{
	fn create_from(
		assets: ResMut<TAssets>,
		storage: ResMut<TStorage>,
	) -> impl GetOrLoadAsset<TAsset> {
		(assets, storage)
	}
}

impl<TAssets, TAsset, TStorage> GetOrLoadAsset<TAsset>
	for (ResMut<'_, TAssets>, ResMut<'_, TStorage>)
where
	TAssets: Resource + LoadAsset,
	TAsset: Asset,
	TStorage: Resource + Storage<Path, Handle<TAsset>>,
{
	fn get_or_load(&mut self, key: Path) -> Handle<TAsset> {
		let (assets, cache) = self;
		cache.get_or_create(key.clone(), || assets.load_asset(key.clone()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;
	use bevy::{
		asset::AssetPath,
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use uuid::Uuid;

	#[derive(Default, Resource)]
	struct _Storage {
		args: Vec<(Path, Handle<Image>)>,
		returns: Handle<Image>,
	}

	impl Storage<Path, Handle<Image>> for _Storage {
		fn get_or_create(
			&mut self,
			key: Path,
			new: impl FnOnce() -> Handle<Image>,
		) -> Handle<Image> {
			self.args.push((key, new()));
			self.returns.clone()
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _LoadAsset {
		mock: Mock_LoadAsset,
	}

	#[automock]
	impl LoadAsset for _LoadAsset {
		fn load_asset<'a, TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	fn setup(load_asset: _LoadAsset) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(load_asset);
		app.init_resource::<_Storage>();

		app
	}

	fn run_in_system(
		app: &mut App,
		mut callback: impl FnMut(ResMut<_LoadAsset>, ResMut<_Storage>) + Send + Sync + 'static,
	) -> Result<(), RunSystemError> {
		app.world_mut().run_system_once(
			move |load_asset: ResMut<_LoadAsset>, storage: ResMut<_Storage>| {
				callback(load_asset, storage);
			},
		)
	}

	#[test]
	fn return_stored_asset() -> Result<(), RunSystemError> {
		let mut app = setup(_LoadAsset::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, Path>()
				.return_const(Handle::default());
		}));
		let stored_asset = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		app.insert_resource(_Storage {
			returns: stored_asset.clone(),
			..default()
		});

		run_in_system(&mut app, move |load_asset, storage| {
			let handle = (load_asset, storage).get_or_load(Path::from(""));
			assert_eq!(stored_asset, handle);
		})
	}

	#[test]
	fn call_storage_with_proper_args() -> Result<(), RunSystemError> {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(_LoadAsset::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, Path>()
				.return_const(handle.clone());
		}));

		run_in_system(&mut app, |load_asset, storage| {
			(load_asset, storage).get_or_load(Path::from("proper path"));
		})?;

		let storage = app.world().resource::<_Storage>();
		assert_eq!(vec![(Path::from("proper path"), handle)], storage.args);
		Ok(())
	}

	#[test]
	fn call_load_asset_with_proper_path() -> Result<(), RunSystemError> {
		let mut app = setup(_LoadAsset::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, Path>()
				.with(eq(Path::from("proper path")))
				.return_const(Handle::default());
		}));

		run_in_system(&mut app, |load_asset, storage| {
			(load_asset, storage).get_or_load(Path::from("proper path"));
		})
	}
}
