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
	use common::traits::load_asset::mock::MockAssetServer;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Debug, PartialEq)]
	struct _Cell;

	fn setup(load_assets: MockAssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(load_assets);
		app.add_observer(load_map_image::<_Cell, MockAssetServer>("my.file"));

		app
	}

	#[test]
	fn load_image() {
		let handle = new_handle();
		let mut app = setup(
			MockAssetServer::default()
				.path("my/path/my.file")
				.returns(handle.clone()),
		);
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
	fn reload_image_when_reinserted() {
		let mut app = setup(MockAssetServer::default());
		app.world_mut()
			.spawn(MapFolder::<_Cell>::from("my/path"))
			.insert(MapFolder::<_Cell>::from("my/other/path"));

		app.update();

		let server = app.world().resource::<MockAssetServer>();
		assert_eq!(
			(1, 1),
			(
				server.calls("my/path/my.file"),
				server.calls("my/other/path/my.file")
			)
		);
	}
}
