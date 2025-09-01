use crate::components::icon::Icon;
use bevy::{asset::LoadState, prelude::*};
use common::traits::{
	get_asset_load_state::GetAssetLoadState,
	load_asset::{AssetNotFound, TryLoadAsset},
};
use std::path::PathBuf;

impl Icon {
	pub(crate) fn load_image(server: ResMut<AssetServer>, icons: Query<&mut Self>) {
		load_icon_image(server, icons);
	}
}

fn load_icon_image<TAssetServer>(mut server: ResMut<TAssetServer>, mut icons: Query<&mut Icon>)
where
	TAssetServer: TryLoadAsset + GetAssetLoadState + Resource,
{
	for mut icon in &mut icons {
		match icon.as_ref() {
			Icon::ImagePath(path) => {
				let path = path.clone();
				let server = server.as_mut();
				set_loading_or_none(&mut icon, server, path);
			}
			Icon::Loading(handle) => {
				let server = server.as_ref();
				let handle = handle.clone();
				set_loaded_or_none(&mut icon, server, handle);
			}
			Icon::Loaded(_) => {}
			Icon::None => {}
		}
	}
}

fn set_loading_or_none<TAssetServer>(icon: &mut Icon, server: &mut TAssetServer, path_buf: PathBuf)
where
	TAssetServer: TryLoadAsset,
{
	*icon = match server.try_load_asset(path_buf) {
		Ok(handle) => Icon::Loading(handle),
		Err(AssetNotFound) => Icon::None,
	};
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
	use bevy::asset::{AssetLoadError, AssetPath, LoadState, UntypedAssetId, io::AssetReaderError};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::{path::PathBuf, sync::Arc};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Resource, NestedMocks)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	impl TryLoadAsset for _AssetServer {
		fn try_load_asset<TAsset, TPath>(
			&mut self,
			path: TPath,
		) -> Result<Handle<TAsset>, AssetNotFound>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.try_load_asset(path)
		}
	}

	impl GetAssetLoadState for _AssetServer {
		fn get_asset_load_state(&self, id: UntypedAssetId) -> Option<LoadState> {
			self.mock.get_asset_load_state(id)
		}
	}

	mock! {
		_AssetServer {}
		impl TryLoadAsset for _AssetServer {
			fn try_load_asset<TAsset, TPath>(
				&mut self,
				path: TPath
			) -> Result<Handle<TAsset>, AssetNotFound>
			where
				TAsset: Asset,
				TPath: Into<AssetPath<'static>> + 'static,;
		}
		impl GetAssetLoadState for _AssetServer {
			fn get_asset_load_state(&self, id: UntypedAssetId) -> Option<LoadState>;
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
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_try_load_asset::<Image, PathBuf>()
				.with(eq(PathBuf::from("my/path")))
				.times(1)
				.return_const(Ok(handle.clone()));
			mock.expect_get_asset_load_state()
				.never()
				.return_const(LoadState::Loaded);
		}));
		let entity = app
			.world_mut()
			.spawn(Icon::ImagePath(PathBuf::from("my/path")))
			.id();

		app.update();

		assert_eq!(
			Some(&Icon::Loading(handle)),
			app.world().entity(entity).get::<Icon>(),
		);
	}

	#[test]
	fn set_image_to_loaded() {
		let handle = new_handle();
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_try_load_asset::<Image, PathBuf>()
				.never()
				.return_const(Ok(handle.clone()));
			mock.expect_get_asset_load_state()
				.with(eq(handle.id().untyped()))
				.times(1)
				.return_const(LoadState::Loaded);
		}));
		let entity = app.world_mut().spawn(Icon::Loading(handle.clone())).id();

		app.update();

		assert_eq!(
			Some(&Icon::Loaded(handle)),
			app.world().entity(entity).get::<Icon>(),
		);
	}

	#[test]
	fn set_image_to_none() {
		let handle = new_handle();
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_try_load_asset::<Image, PathBuf>()
				.never()
				.return_const(Ok(handle.clone()));
			mock.expect_get_asset_load_state()
				.with(eq(handle.id().untyped()))
				.times(1)
				.return_const(LoadState::Failed(Arc::new(
					AssetLoadError::AssetReaderError(AssetReaderError::NotFound(PathBuf::from(""))),
				)));
		}));
		let entity = app.world_mut().spawn(Icon::Loading(handle)).id();

		app.update();

		assert_eq!(Some(&Icon::None), app.world().entity(entity).get::<Icon>());
	}

	#[test]
	fn set_to_none_if_asset_does_not_exist() {
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_try_load_asset::<Image, PathBuf>()
				.with(eq(PathBuf::from("my/path")))
				.times(1)
				.return_const(Err(AssetNotFound));
			mock.expect_get_asset_load_state()
				.never()
				.return_const(LoadState::Loaded);
		}));
		let entity = app
			.world_mut()
			.spawn(Icon::ImagePath(PathBuf::from("my/path")))
			.id();

		app.update();

		assert_eq!(Some(&Icon::None), app.world().entity(entity).get::<Icon>());
	}
}
