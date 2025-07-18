use crate::resources::map::color_lookup::MapColorLookupImage;
use bevy::prelude::*;
use common::traits::{load_asset::LoadAsset, thread_safe::ThreadSafe};
use std::path::PathBuf;

impl<TCell> MapColorLookupImage<TCell>
where
	TCell: ThreadSafe + ColorLookupAssetPath,
{
	pub(crate) fn lookup_images(commands: Commands, asset_server: ResMut<AssetServer>) {
		lookup_images::<TCell, AssetServer>(commands, asset_server)
	}
}

pub(crate) trait ColorLookupAssetPath {
	const LOOKUP_ROOT: &str;
}

fn lookup_images<TCell, TAssetServer>(
	mut commands: Commands,
	mut asset_server: ResMut<TAssetServer>,
) where
	TAssetServer: Resource + LoadAsset,
	TCell: ThreadSafe + ColorLookupAssetPath,
{
	let path = PathBuf::from(TCell::LOOKUP_ROOT)
		.join("floor")
		.with_extension("png");
	let floor = asset_server.load_asset(path);
	commands.insert_resource(MapColorLookupImage::<TCell>::new(floor));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetPath;
	use common::traits::load_asset::LoadAsset;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::path::PathBuf;
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Resource, NestedMocks)]
	struct _Server {
		mock: Mock_Server,
	}

	#[automock]
	impl LoadAsset for _Server {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Cell;

	impl ColorLookupAssetPath for _Cell {
		const LOOKUP_ROOT: &str = "my/root/path";
	}

	fn setup(server: _Server) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(server);
		app.add_systems(Update, lookup_images::<_Cell, _Server>);

		app
	}

	#[test]
	fn set_floor() {
		let floor = new_handle();
		let floor_clone = floor.clone();
		let mut app = setup(_Server::new().with_mock(move |mock| {
			mock.expect_load_asset::<Image, PathBuf>()
				.return_const(floor_clone.clone());
		}));

		app.update();

		assert_eq!(
			Some(&MapColorLookupImage::new(floor)),
			app.world().get_resource::<MapColorLookupImage<_Cell>>(),
		);
	}

	#[test]
	fn load_correct_floor_asset() {
		let mut app = setup(_Server::new().with_mock(move |mock| {
			mock.expect_load_asset::<Image, PathBuf>()
				.with(eq(PathBuf::from("my/root/path/floor.png")))
				.return_const(new_handle());
		}));

		app.update();
	}
}
