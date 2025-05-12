use crate::components::icon::{Icon, IconImage};
use bevy::{asset::LoadState, prelude::*};
use common::traits::{get_asset_load_state::GetAssetLoadState, load_asset::LoadAsset};
use std::path::PathBuf;

impl Icon {
	pub(crate) fn load_image(
		commands: Commands,
		server: ResMut<AssetServer>,
		icons: Query<(Entity, &mut Icon)>,
	) {
		load_icon_image(commands, server, icons);
	}
}

fn load_icon_image<TAssetServer>(
	mut commands: Commands,
	mut server: ResMut<TAssetServer>,
	mut icons: Query<(Entity, &mut Icon)>,
) where
	TAssetServer: LoadAsset + GetAssetLoadState + Resource,
{
	for (entity, mut icon) in &mut icons {
		match &icon.image {
			IconImage::Path(path) => {
				let path = path.clone();
				let commands = &mut commands;
				let server = server.as_mut();
				load_image(&mut icon, commands, server, path, entity);
			}
			IconImage::Loading(handle) => {
				let server = server.as_ref();
				let handle = handle.clone();
				set_loaded_or_none(&mut icon, server, handle);
			}
			IconImage::Loaded(_) => {}
			IconImage::None => {}
		}
	}
}

fn load_image<TAssetServer>(
	icon: &mut Icon,
	commands: &mut Commands<'_, '_>,
	server: &mut TAssetServer,
	path_buf: PathBuf,
	entity: Entity,
) where
	TAssetServer: LoadAsset,
{
	let Some(mut entity) = commands.get_entity(entity) else {
		return;
	};
	let handle = server.load_asset(path_buf.clone());
	entity.try_insert(ImageNode::new(handle.clone()));
	icon.image = IconImage::Loading(handle);
}

fn set_loaded_or_none<TAssetServer>(icon: &mut Icon, server: &TAssetServer, handle: Handle<Image>)
where
	TAssetServer: GetAssetLoadState,
{
	match server.get_asset_load_state(handle.id().untyped()) {
		Some(LoadState::Loaded) => icon.image = IconImage::Loaded(handle),
		Some(LoadState::Failed(_)) => icon.image = IconImage::None,
		_ => {}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::icon::IconImage;
	use bevy::asset::{AssetLoadError, AssetPath, LoadState, UntypedAssetId, io::AssetReaderError};
	use common::{
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::{handles_localization::localized::Localized, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::{path::PathBuf, sync::Arc};

	#[derive(Resource, NestedMocks)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	impl LoadAsset for _AssetServer {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	impl GetAssetLoadState for _AssetServer {
		fn get_asset_load_state(&self, id: UntypedAssetId) -> Option<LoadState> {
			self.mock.get_asset_load_state(id)
		}
	}

	mock! {
		_AssetServer {}
		impl LoadAsset for _AssetServer {
			fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
			where
				TAsset: Asset,
				TPath: Into<AssetPath<'static>> + 'static;
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
	fn load_asset() {
		let handle = new_handle();
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, PathBuf>()
				.with(eq(PathBuf::from("my/path")))
				.times(1)
				.return_const(handle.clone());
			mock.expect_get_asset_load_state()
				.never()
				.return_const(LoadState::Loaded);
		}));
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from(""),
				image: IconImage::Path(PathBuf::from("my/path")),
			})
			.id();

		app.update();

		assert_eq!(
			Some(&handle),
			app.world()
				.entity(entity)
				.get::<ImageNode>()
				.map(|node| &node.image)
		);
	}

	#[test]
	fn set_to_loading() {
		let handle = new_handle();
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, PathBuf>()
				.return_const(handle.clone());
			mock.expect_get_asset_load_state()
				.never()
				.return_const(LoadState::Loaded);
		}));
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from(""),
				image: IconImage::Path(PathBuf::from("")),
			})
			.id();

		app.update();

		assert_eq!(
			Some(&IconImage::Loading(handle)),
			app.world()
				.entity(entity)
				.get::<Icon>()
				.map(|icon| &icon.image)
		);
	}

	#[test]
	fn set_image_to_loaded() {
		let handle = new_handle();
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, PathBuf>()
				.never()
				.return_const(handle.clone());
			mock.expect_get_asset_load_state()
				.with(eq(handle.id().untyped()))
				.times(1)
				.return_const(LoadState::Loaded);
		}));
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from(""),
				image: IconImage::Loading(handle.clone()),
			})
			.id();

		app.update();

		assert_eq!(
			Some(&IconImage::Loaded(handle)),
			app.world()
				.entity(entity)
				.get::<Icon>()
				.map(|icon| &icon.image)
		);
	}

	#[test]
	fn set_image_to_none() {
		let handle = new_handle();
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, PathBuf>()
				.never()
				.return_const(handle.clone());
			mock.expect_get_asset_load_state()
				.with(eq(handle.id().untyped()))
				.times(1)
				.return_const(LoadState::Failed(Arc::new(
					AssetLoadError::AssetReaderError(AssetReaderError::NotFound(PathBuf::from(""))),
				)));
		}));
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from(""),
				image: IconImage::Loading(handle),
			})
			.id();

		app.update();

		assert_eq!(
			Some(&IconImage::None),
			app.world()
				.entity(entity)
				.get::<Icon>()
				.map(|icon| &icon.image)
		);
	}
}
