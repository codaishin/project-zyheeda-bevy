use super::begin_loading_resource::AssetResourceHandle;
use bevy::prelude::*;

impl<T> InstantiateResource for T where T: Resource + Asset {}

pub(crate) trait InstantiateResource: Resource + Asset + Sized {
	fn instantiate(
		mut commands: Commands,
		resource_handle: Res<AssetResourceHandle<Self>>,
		mut assets: ResMut<Assets<Self>>,
	) {
		let AssetResourceHandle(handle) = resource_handle.as_ref();
		let Some(resource) = assets.remove(handle) else {
			return;
		};

		commands.remove_resource::<AssetResourceHandle<Self>>();
		commands.insert_resource(resource);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::{SingleThreadedApp, new_handle};

	#[derive(Resource, Asset, TypePath, Debug, PartialEq)]
	struct _Asset;

	fn setup<const N: usize>(
		assets: [(&Handle<_Asset>, _Asset); N],
		handle: AssetResourceHandle<_Asset>,
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets_resource = Assets::default();

		for (handle, asset) in assets {
			assets_resource.insert(handle, asset);
		}

		app.insert_resource(handle);
		app.insert_resource(assets_resource);
		app.add_systems(Update, _Asset::instantiate);

		app
	}

	#[test]
	fn insert_asset() {
		let handle = new_handle();
		let assets = [(&handle, _Asset)];
		let mut app = setup(assets, AssetResourceHandle(handle.clone()));

		app.update();

		assert_eq!(Some(&_Asset), app.world().get_resource::<_Asset>());
	}

	#[test]
	fn remove_handle_cache() {
		let handle = new_handle();
		let assets = [(&handle, _Asset)];
		let mut app = setup(assets, AssetResourceHandle(handle.clone()));

		app.update();

		assert_eq!(
			None,
			app.world().get_resource::<AssetResourceHandle<_Asset>>()
		);
	}

	#[test]
	fn do_not_remove_handle_cache_if_asset_cannot_be_found() {
		let handle = new_handle();
		let assets = [(&new_handle(), _Asset)];
		let mut app = setup(assets, AssetResourceHandle(handle.clone()));

		app.update();

		assert_eq!(
			Some(&AssetResourceHandle(handle.clone())),
			app.world().get_resource::<AssetResourceHandle<_Asset>>()
		);
	}
}
