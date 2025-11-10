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
	use common::traits::load_asset::mock::MockAssetServer;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Debug, PartialEq)]
	struct _Cell;

	impl ColorLookupAssetPath for _Cell {
		const LOOKUP_ROOT: &str = "my/root/path";
	}

	fn setup(server: MockAssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(server);
		app.add_systems(Update, lookup_images::<_Cell, MockAssetServer>);

		app
	}

	#[test]
	fn set_floor() {
		let floor = new_handle();
		let mut app = setup(
			MockAssetServer::default()
				.path(format!("{}/floor.png", _Cell::LOOKUP_ROOT))
				.returns(floor.clone()),
		);

		app.update();

		assert_eq!(
			Some(&MapColorLookupImage::new(floor)),
			app.world().get_resource::<MapColorLookupImage<_Cell>>(),
		);
	}
}
