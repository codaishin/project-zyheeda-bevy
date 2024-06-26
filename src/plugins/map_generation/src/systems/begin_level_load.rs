use crate::{components::LoadLevelCommand, map::Map, traits::SourcePath};
use bevy::{
	ecs::system::{Commands, Resource},
	prelude::ResMut,
	reflect::TypePath,
};
use common::traits::load_asset::LoadAsset;

pub(crate) fn begin_level_load<
	TLoadMap: LoadAsset<Map<TCell>> + Resource,
	TCell: SourcePath + TypePath + Sync + Send,
>(
	mut commands: Commands,
	mut map_loader: ResMut<TLoadMap>,
) {
	let map = map_loader.load_asset(TCell::source_path());
	commands.insert_resource(LoadLevelCommand(map));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::LoadLevelCommand, map::Map};
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Handle},
		reflect::TypePath,
		utils::Uuid,
	};
	use common::{test_tools::utils::SingleThreadedApp, traits::load_asset::Path};
	use mockall::{automock, predicate::eq};

	#[derive(TypePath, Asset, Debug, PartialEq)]
	struct _Cell;

	impl SourcePath for _Cell {
		fn source_path() -> Path {
			Path::from("aaa/bbb/ccc.file_format")
		}
	}

	#[derive(Resource, Default)]
	struct _LoadMap {
		mock: Mock_LoadMap,
	}

	#[automock]
	impl LoadAsset<Map<_Cell>> for _LoadMap {
		fn load_asset(&mut self, path: Path) -> Handle<Map<_Cell>> {
			self.mock.load_asset(path)
		}
	}

	fn setup(load_map: _LoadMap) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(load_map);
		app.add_systems(Update, begin_level_load::<_LoadMap, _Cell>);

		app
	}

	#[test]
	fn insert_level_command() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut load_map = _LoadMap::default();
		load_map
			.mock
			.expect_load_asset()
			.times(1)
			.with(eq(Path::from("aaa/bbb/ccc.file_format")))
			.return_const(handle.clone());
		let mut app = setup(load_map);

		app.update();

		let level_command = app.world.get_resource::<LoadLevelCommand<_Cell>>();

		assert_eq!(Some(&LoadLevelCommand(handle)), level_command);
	}
}
