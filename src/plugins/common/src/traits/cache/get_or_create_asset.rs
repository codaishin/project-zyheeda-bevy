use super::{Cache, GetOrCreateAsset};
use crate::traits::add_asset::AddAsset;
use bevy::{
	asset::{Asset, Handle},
	prelude::{ResMut, Resource},
};

impl<TAssets, TAsset, TCache, TKey> GetOrCreateAsset<TKey, TAsset>
	for (ResMut<'_, TAssets>, ResMut<'_, TCache>)
where
	TAssets: Resource + AddAsset<TAsset>,
	TAsset: Asset,
	TCache: Resource + Cache<TKey, Handle<TAsset>>,
{
	fn get_or_create(&mut self, key: TKey, create: impl FnOnce() -> TAsset) -> Handle<TAsset> {
		let (assets, cache) = self;
		cache.cached(key, || assets.add_asset(create()))
	}
}

#[cfg(test)]
mod tests {
	use crate::test_tools::utils::SingleThreadedApp;

	use super::*;
	use bevy::{
		app::{App, Update},
		asset::AssetId,
		pbr::StandardMaterial,
		utils::{default, Uuid},
	};

	#[derive(Default, Resource)]
	struct _Cache {
		args: Vec<(u32, Handle<StandardMaterial>)>,
		returns: Handle<StandardMaterial>,
	}

	impl Cache<u32, Handle<StandardMaterial>> for _Cache {
		fn cached(
			&mut self,
			key: u32,
			new: impl FnOnce() -> Handle<StandardMaterial>,
		) -> Handle<StandardMaterial> {
			self.args.push((key, new()));
			self.returns.clone()
		}
	}

	#[derive(Default, Resource)]
	struct _Assets {
		args: Vec<StandardMaterial>,
		returns: Handle<StandardMaterial>,
	}

	impl AddAsset<StandardMaterial> for _Assets {
		fn add_asset(&mut self, asset: StandardMaterial) -> Handle<StandardMaterial> {
			self.args.push(asset);
			self.returns.clone()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Assets>();
		app.init_resource::<_Cache>();

		app
	}

	fn run_system(
		app: &mut App,
		mut callback: impl FnMut(ResMut<_Assets>, ResMut<_Cache>) + Send + Sync + 'static,
	) {
		app.add_systems(
			Update,
			move |assets: ResMut<_Assets>, cache: ResMut<_Cache>| {
				callback(assets, cache);
			},
		);
		app.update();
	}

	#[test]
	fn return_cached_asset() {
		let cached_asset = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup();

		app.world.insert_resource(_Cache {
			returns: cached_asset.clone(),
			..default()
		});

		run_system(&mut app, move |assets, cache| {
			let handle = (assets, cache).get_or_create(0, StandardMaterial::default);
			assert_eq!(cached_asset, handle);
		});
	}

	#[test]
	fn call_cached_with_proper_args() {
		let asset_handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup();

		app.insert_resource(_Assets {
			returns: asset_handle.clone(),
			..default()
		});

		run_system(&mut app, |assets, cache| {
			(assets, cache).get_or_create(42, StandardMaterial::default);
		});

		let cache = app.world.resource::<_Cache>();
		assert_eq!(vec![(42, asset_handle)], cache.args);
	}
}
