use crate::{components::LoadLevelCommand, map::Map};
use bevy::ecs::system::{Commands, Res, Resource};
use common::traits::load_asset::LoadAsset;

pub(crate) fn begin_level_load<TLoadMap: LoadAsset<Map> + Resource>(
	mut commands: Commands,
	map_loader: Res<TLoadMap>,
) {
	let map = map_loader.load_asset("maps/map.txt");
	commands.insert_resource(LoadLevelCommand(map));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::LoadLevelCommand, map::Map};
	use bevy::{
		app::{App, Update},
		asset::{AssetId, AssetPath, Handle},
		utils::Uuid,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use std::collections::HashMap;

	#[derive(Resource, Default)]
	struct _LoadMap(HashMap<String, Handle<Map>>);

	impl LoadAsset<Map> for _LoadMap {
		fn load_asset<'a, TPath: Into<AssetPath<'a>>>(&self, path: TPath) -> Handle<Map> {
			let path: AssetPath = path.into();
			self.0
				.iter()
				.find_map(|(key, value)| match AssetPath::from(key) == path {
					true => Some(value.clone()),
					false => None,
				})
				.unwrap_or(Handle::<Map>::Weak(AssetId::Uuid {
					uuid: Uuid::new_v4(),
				}))
				.clone()
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
		let mut app = setup(_LoadMap(
			[("maps/map.txt".to_owned(), handle.clone())].into(),
		));

		app.update();

		let level_command = app.world.get_resource::<LoadLevelCommand<Map>>();

		assert_eq!(Some(&LoadLevelCommand(handle)), level_command);
	}
}
