use super::{Cache, GetOrLoadAsset};
use crate::traits::load_asset::{LoadAsset, Path};
use bevy::{
	asset::{Asset, Handle},
	prelude::{ResMut, Resource},
};

impl<TAssets, TAsset, TCache> GetOrLoadAsset<TAsset> for (ResMut<'_, TAssets>, ResMut<'_, TCache>)
where
	TAssets: Resource + LoadAsset<TAsset>,
	TAsset: Asset,
	TCache: Resource + Cache<Path, Handle<TAsset>>,
{
	fn get_or_load(&mut self, key: Path) -> Handle<TAsset> {
		let (assets, cache) = self;
		cache.cached(key.clone(), || assets.load_asset(key))
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
		utils::Uuid,
	};

	#[derive(Default, Resource)]
	struct _Cache {
		args: Vec<(Path, Handle<Image>)>,
		returns: Handle<Image>,
	}

	impl Cache<Path, Handle<Image>> for _Cache {
		fn cached(&mut self, key: Path, new: impl FnOnce() -> Handle<Image>) -> Handle<Image> {
			self.args.push((key, new()));
			self.returns.clone()
		}
	}

	#[derive(Default, Resource)]
	struct _Assets {
		args: Vec<Path>,
		returns: Handle<Image>,
	}

	impl LoadAsset<Image> for _Assets {
		fn load_asset(&mut self, path: Path) -> Handle<Image> {
			self.args.push(path);
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

		app.insert_resource(_Cache {
			returns: cached_asset.clone(),
			..default()
		});

		run_system(&mut app, move |assets, cache| {
			let handle = (assets, cache).get_or_load(Path::from(""));
			assert_eq!(cached_asset, handle);
		})
	}

	#[test]
	fn call_cached_with_proper_args() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup();

		app.insert_resource(_Assets {
			returns: handle.clone(),
			..default()
		});

		run_system(&mut app, |assets, cache| {
			(assets, cache).get_or_load(Path::from("proper path"));
		});

		let cache = app.world.resource::<_Cache>();
		assert_eq!(vec![(Path::from("proper path"), handle)], cache.args);
	}

	#[test]
	fn call_load_asset_with_proper_path() {
		let mut app = setup();

		run_system(&mut app, |assets, cache| {
			(assets, cache).get_or_load(Path::from("proper path"));
		});

		let assets = app.world.resource::<_Assets>();
		assert_eq!(vec![Path::from("proper path")], assets.args);
	}
}
