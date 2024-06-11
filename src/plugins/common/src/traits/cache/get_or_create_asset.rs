use super::{GetOrCreateAsset, GetOrCreateAssetFactory, Storage};
use crate::{tools::Factory, traits::add_asset::AddAsset};
use bevy::{
	asset::{Asset, Handle},
	prelude::{ResMut, Resource},
};

pub struct CreateAssetCache;

impl<TAssets, TAsset, TStorage, TKey> GetOrCreateAssetFactory<TAssets, TAsset, TStorage, TKey>
	for Factory<CreateAssetCache>
where
	TAssets: Resource + AddAsset<TAsset>,
	TAsset: Asset,
	TStorage: Resource + Storage<TKey, Handle<TAsset>>,
{
	fn create_from(
		assets: ResMut<TAssets>,
		storage: ResMut<TStorage>,
	) -> impl GetOrCreateAsset<TKey, TAsset> {
		(assets, storage)
	}
}

impl<TAssets, TAsset, TStorage, TKey> GetOrCreateAsset<TKey, TAsset>
	for (ResMut<'_, TAssets>, ResMut<'_, TStorage>)
where
	TAssets: Resource + AddAsset<TAsset>,
	TAsset: Asset,
	TStorage: Resource + Storage<TKey, Handle<TAsset>>,
{
	fn get_or_create(&mut self, key: TKey, mut create: impl FnMut() -> TAsset) -> Handle<TAsset> {
		let (assets, cache) = self;
		cache.get_or_create(key, || assets.add_asset(create()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;
	use bevy::{
		app::{App, Update},
		asset::AssetId,
		pbr::StandardMaterial,
		utils::{default, Uuid},
	};

	#[derive(Default, Resource)]
	struct _Storage {
		args: Vec<(u32, Handle<StandardMaterial>)>,
		returns: Handle<StandardMaterial>,
	}

	impl Storage<u32, Handle<StandardMaterial>> for _Storage {
		fn get_or_create(
			&mut self,
			key: u32,
			new: impl FnOnce() -> Handle<StandardMaterial>,
		) -> Handle<StandardMaterial> {
			self.args.push((key, new()));
			self.returns.clone()
		}
	}

	#[derive(Default, Resource)]
	struct _AddAsset {
		args: Vec<StandardMaterial>,
		returns: Handle<StandardMaterial>,
	}

	impl AddAsset<StandardMaterial> for _AddAsset {
		fn add_asset(&mut self, asset: StandardMaterial) -> Handle<StandardMaterial> {
			self.args.push(asset);
			self.returns.clone()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_AddAsset>();
		app.init_resource::<_Storage>();

		app
	}

	fn run_system(
		app: &mut App,
		mut callback: impl FnMut(ResMut<_AddAsset>, ResMut<_Storage>) + Send + Sync + 'static,
	) {
		app.add_systems(
			Update,
			move |add_asset: ResMut<_AddAsset>, storage: ResMut<_Storage>| {
				callback(add_asset, storage);
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

		app.world.insert_resource(_Storage {
			returns: stored_asset.clone(),
			..default()
		});

		run_system(&mut app, move |add_asset, storage| {
			let handle = (add_asset, storage).get_or_create(0, StandardMaterial::default);
			assert_eq!(stored_asset, handle);
		});
	}

	#[test]
	fn call_storage_with_proper_args() {
		let asset_handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup();

		app.insert_resource(_AddAsset {
			returns: asset_handle.clone(),
			..default()
		});

		run_system(&mut app, |add_asset, storage| {
			(add_asset, storage).get_or_create(42, StandardMaterial::default);
		});

		let storage = app.world.resource::<_Storage>();
		assert_eq!(vec![(42, asset_handle)], storage.args);
	}
}
