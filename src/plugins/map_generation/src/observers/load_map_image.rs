use crate::components::{map_asset::MapAsset, map_image::MapImage};
use bevy::prelude::*;
use common::traits::{load_asset::LoadAsset, thread_safe::ThreadSafe, try_insert_on::TryInsertOn};

impl<TCell> MapAsset<TCell>
where
	TCell: ThreadSafe,
{
	pub(crate) fn load_map_image(
		trigger: Trigger<OnInsert, Self>,
		commands: Commands,
		asset_server: ResMut<AssetServer>,
		assets: Query<&Self>,
	) {
		load_map_image(trigger, commands, asset_server, assets)
	}
}

fn load_map_image<TCell, TAssets>(
	trigger: Trigger<OnInsert, MapAsset<TCell>>,
	mut commands: Commands,
	mut asset_server: ResMut<TAssets>,
	assets: Query<&MapAsset<TCell>>,
) where
	TCell: ThreadSafe,
	TAssets: Resource + LoadAsset,
{
	let entity = trigger.target();
	let Ok(MapAsset { path, .. }) = assets.get(entity) else {
		return;
	};
	let handle = asset_server.load_asset(path.clone());
	commands.try_insert_on(entity, MapImage::<TCell>::from(handle));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::map_image::MapImage;
	use bevy::asset::AssetPath;
	use common::traits::load_asset::Path;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Debug, PartialEq)]
	struct _Cell;

	#[derive(Resource, NestedMocks)]
	struct _Assets {
		mock: Mock_Assets,
	}

	#[automock]
	impl LoadAsset for _Assets {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	fn setup(load_assets: _Assets) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(load_assets);
		app.add_observer(load_map_image::<_Cell, _Assets>);

		app
	}

	#[test]
	fn load_image() {
		let handle = new_handle();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, Path>()
				.return_const(handle.clone());
		}));
		let entity = app
			.world_mut()
			.spawn(MapAsset::<_Cell>::from("my/path"))
			.id();

		app.update();

		assert_eq!(
			Some(&MapImage::from(handle)),
			app.world().entity(entity).get::<MapImage<_Cell>>(),
		);
	}

	#[test]
	fn use_correct_path() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, Path>()
				.with(eq(Path::from("my/path")))
				.times(1)
				.return_const(new_handle());
		}));
		app.world_mut().spawn(MapAsset::<_Cell>::from("my/path"));

		app.update();
	}

	#[test]
	fn reload_image_when_reinserted() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, Path>()
				.with(eq(Path::from("my/path")))
				.times(1)
				.return_const(new_handle());
			mock.expect_load_asset::<Image, Path>()
				.with(eq(Path::from("my/other/path")))
				.times(1)
				.return_const(new_handle());
		}));
		app.world_mut()
			.spawn(MapAsset::<_Cell>::from("my/path"))
			.insert(MapAsset::<_Cell>::from("my/other/path"));

		app.update();
	}
}
