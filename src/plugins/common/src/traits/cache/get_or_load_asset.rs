use super::{GetOrLoadAsset, GetOrLoadAssetFactory, Storage};
use crate::{
	tools::Factory,
	traits::load_asset::{LoadAsset, Path},
};
use bevy::{
	asset::{Asset, Handle},
	prelude::{ResMut, Resource},
};

pub struct LoadAssetCache;

impl<TAssets, TAsset, TStorage> GetOrLoadAssetFactory<TAssets, TAsset, TStorage>
	for Factory<LoadAssetCache>
where
	TAssets: Resource + LoadAsset<TAsset>,
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
	TAssets: Resource + LoadAsset<TAsset>,
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
		app::{App, Update},
		asset::AssetId,
		prelude::default,
		render::texture::Image,
	};
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

	#[derive(Default, Resource)]
	struct _LoadAsset {
		args: Vec<Path>,
		returns: Handle<Image>,
	}

	impl LoadAsset<Image> for _LoadAsset {
		fn load_asset(&mut self, path: Path) -> Handle<Image> {
			self.args.push(path);
			self.returns.clone()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_LoadAsset>();
		app.init_resource::<_Storage>();

		app
	}

	fn run_system(
		app: &mut App,
		mut callback: impl FnMut(ResMut<_LoadAsset>, ResMut<_Storage>) + Send + Sync + 'static,
	) {
		app.add_systems(
			Update,
			move |load_asset: ResMut<_LoadAsset>, storage: ResMut<_Storage>| {
				callback(load_asset, storage);
			},
		);
		app.update();
	}

	#[test]
	fn return_stored_asset() {
		let stored_asset = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup();

		app.insert_resource(_Storage {
			returns: stored_asset.clone(),
			..default()
		});

		run_system(&mut app, move |load_asset, storage| {
			let handle = (load_asset, storage).get_or_load(Path::from(""));
			assert_eq!(stored_asset, handle);
		})
	}

	#[test]
	fn call_storage_with_proper_args() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup();

		app.insert_resource(_LoadAsset {
			returns: handle.clone(),
			..default()
		});

		run_system(&mut app, |load_asset, storage| {
			(load_asset, storage).get_or_load(Path::from("proper path"));
		});

		let storage = app.world().resource::<_Storage>();
		assert_eq!(vec![(Path::from("proper path"), handle)], storage.args);
	}

	#[test]
	fn call_load_asset_with_proper_path() {
		let mut app = setup();

		run_system(&mut app, |load_asset, storage| {
			(load_asset, storage).get_or_load(Path::from("proper path"));
		});

		let load_asset = app.world().resource::<_LoadAsset>();
		assert_eq!(vec![Path::from("proper path")], load_asset.args);
	}
}
