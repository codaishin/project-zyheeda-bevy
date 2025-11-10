use crate::components::icon::Icon;
use bevy::{asset::LoadState, prelude::*};
use common::traits::{get_asset_load_state::GetAssetLoadState, load_asset::LoadAsset};
use std::path::PathBuf;

impl Icon {
	pub(crate) fn load_image(server: ResMut<AssetServer>, icons: Query<&mut Self>) {
		load_icon_image(server, icons);
	}
}

fn load_icon_image<TAssetServer>(mut server: ResMut<TAssetServer>, mut icons: Query<&mut Icon>)
where
	TAssetServer: LoadAsset + GetAssetLoadState + Resource,
{
	for mut icon in &mut icons {
		match icon.as_ref() {
			Icon::ImagePath(path) => {
				let path = path.clone();
				let server = server.as_mut();
				set_loading(&mut icon, server, path);
			}
			Icon::Load(handle) => {
				let server = server.as_ref();
				let handle = handle.clone();
				set_loaded_or_none(&mut icon, server, handle);
			}
			Icon::Loaded(_) => {}
			Icon::None => {}
		}
	}
}

fn set_loading<TAssetServer>(icon: &mut Icon, server: &mut TAssetServer, path_buf: PathBuf)
where
	TAssetServer: LoadAsset,
{
	*icon = Icon::Load(server.load_asset(path_buf));
}

fn set_loaded_or_none<TAssetServer>(icon: &mut Icon, server: &TAssetServer, handle: Handle<Image>)
where
	TAssetServer: GetAssetLoadState,
{
	match server.get_asset_load_state(handle.id().untyped()) {
		Some(LoadState::Loaded) => *icon = Icon::Loaded(handle),
		Some(LoadState::Failed(_)) => *icon = Icon::None,
		_ => {}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		asset::{AssetLoadError, AssetPath, LoadState, UntypedAssetId, io::AssetReaderError},
		platform::collections::HashMap,
	};
	use common::traits::load_asset::mock::MockAssetServer;
	use std::{path::PathBuf, sync::Arc};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Resource, Default)]
	struct _AssetServer {
		mock: MockAssetServer,
		load_states: HashMap<UntypedAssetId, LoadState>,
	}

	impl LoadAsset for _AssetServer {
		fn load_asset<'a, TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'a>>,
		{
			self.mock.load_asset(path)
		}
	}

	impl GetAssetLoadState for _AssetServer {
		fn get_asset_load_state(&self, id: UntypedAssetId) -> Option<LoadState> {
			self.load_states.get(&id).cloned()
		}
	}

	fn setup(server: _AssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(server);
		app.add_systems(Update, load_icon_image::<_AssetServer>);

		app
	}

	#[test]
	fn set_to_loading() {
		let handle = new_handle();
		let mut app = setup(_AssetServer {
			mock: MockAssetServer::default()
				.path("my/path")
				.returns(handle.clone()),
			..default()
		});
		let entity = app
			.world_mut()
			.spawn(Icon::ImagePath(PathBuf::from("my/path")))
			.id();

		app.update();

		assert_eq!(
			Some(&Icon::Load(handle)),
			app.world().entity(entity).get::<Icon>(),
		);
	}

	#[test]
	fn set_image_to_loaded() {
		let handle = new_handle();
		let mut app = setup(_AssetServer {
			load_states: HashMap::from([(handle.id().untyped(), LoadState::Loaded)]),
			..default()
		});
		let entity = app.world_mut().spawn(Icon::Load(handle.clone())).id();

		app.update();

		assert_eq!(
			Some(&Icon::Loaded(handle)),
			app.world().entity(entity).get::<Icon>(),
		);
	}

	#[test]
	fn set_image_to_none() {
		let handle = new_handle();
		let mut app = setup(_AssetServer {
			load_states: HashMap::from([(
				handle.id().untyped(),
				LoadState::Failed(Arc::new(AssetLoadError::AssetReaderError(
					AssetReaderError::NotFound(PathBuf::from("")),
				))),
			)]),
			..default()
		});
		let entity = app.world_mut().spawn(Icon::Load(handle)).id();

		app.update();

		assert_eq!(Some(&Icon::None), app.world().entity(entity).get::<Icon>());
	}
}
