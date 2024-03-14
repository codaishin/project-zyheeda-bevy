use crate::{components::LoadLevelCommand, traits::LoadMap};
use bevy::ecs::system::{Commands, Res, Resource};

pub(crate) fn begin_level_load<TLoadMap: LoadMap + Resource>(
	mut commands: Commands,
	map_loader: Res<TLoadMap>,
) {
	let map = map_loader.load();
	commands.insert_resource(LoadLevelCommand(map));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::LoadLevelCommand, map::Map};
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		utils::Uuid,
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Resource, Default)]
	struct _LoadMap(Handle<Map>);

	impl LoadMap for _LoadMap {
		fn load(&self) -> Handle<Map> {
			self.0.clone()
		}
	}

	fn setup(load_map: _LoadMap) -> App {
		let mut app = App::new_single_threaded([Update]);
		app.insert_resource(load_map);
		app.add_systems(Update, begin_level_load::<_LoadMap>);

		app
	}

	#[test]
	fn insert_level_command() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut app = setup(_LoadMap(handle.clone()));

		app.update();

		let level_command = app.world.get_resource::<LoadLevelCommand<Map>>();

		assert_eq!(Some(&LoadLevelCommand(handle)), level_command);
	}
}
