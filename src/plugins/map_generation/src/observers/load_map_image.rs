use crate::components::map::{folder::MapFolder, image::MapImage};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, load_asset::LoadAsset, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaCommands,
};

impl<TCell> MapFolder<TCell>
where
	TCell: ThreadSafe,
{
	pub(crate) fn load_map_image(
		file: &str,
	) -> impl Fn(Trigger<OnInsert, Self>, ZyheedaCommands, ResMut<AssetServer>, Query<&Self>) {
		load_map_image(file)
	}
}

#[allow(clippy::type_complexity)]
fn load_map_image<TCell, TAssets>(
	file: &str,
) -> impl Fn(
	Trigger<OnInsert, MapFolder<TCell>>,
	ZyheedaCommands,
	ResMut<TAssets>,
	Query<&MapFolder<TCell>>,
)
where
	TCell: ThreadSafe,
	TAssets: Resource + LoadAsset,
{
	move |trigger, mut commands, mut asset_server, assets| {
		let entity = trigger.target();
		let Ok(MapFolder { path, .. }) = assets.get(entity) else {
			return;
		};
		let handle = asset_server.load_asset(path.join(file));
		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(MapImage::<TCell>::from(handle));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetPath;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::path::PathBuf;
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
		app.add_observer(load_map_image::<_Cell, _Assets>("my.file"));

		app
	}

	#[test]
	fn load_image() {
		let handle = new_handle();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, PathBuf>()
				.return_const(handle.clone());
		}));
		let entity = app
			.world_mut()
			.spawn(MapFolder::<_Cell>::from("my/path"))
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
			mock.expect_load_asset::<Image, PathBuf>()
				.with(eq(PathBuf::from("my/path").join("my.file")))
				.times(1)
				.return_const(new_handle());
		}));
		app.world_mut().spawn(MapFolder::<_Cell>::from("my/path"));

		app.update();
	}

	#[test]
	fn reload_image_when_reinserted() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Image, PathBuf>()
				.with(eq(PathBuf::from("my/path").join("my.file")))
				.times(1)
				.return_const(new_handle());
			mock.expect_load_asset::<Image, PathBuf>()
				.with(eq(PathBuf::from("my/other/path").join("my.file")))
				.times(1)
				.return_const(new_handle());
		}));
		app.world_mut()
			.spawn(MapFolder::<_Cell>::from("my/path"))
			.insert(MapFolder::<_Cell>::from("my/other/path"));

		app.update();
	}
}
