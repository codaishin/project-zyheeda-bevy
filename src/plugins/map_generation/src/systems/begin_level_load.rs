use crate::{components::LoadLevelCommand, map::Map};
use bevy::{
	ecs::system::{Commands, Res, Resource},
	reflect::TypePath,
};
use common::traits::load_asset::LoadAsset;

pub(crate) fn begin_level_load<
	TLoadMap: LoadAsset<Map<TCell>> + Resource,
	TCell: TypePath + Sync + Send,
>(
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
		asset::{Asset, AssetId, AssetPath, Handle},
		reflect::TypePath,
		utils::Uuid,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use std::collections::HashMap;

	#[derive(TypePath, Asset, Debug, PartialEq)]
	struct _Cell;

	#[derive(Resource, Default)]
	struct _LoadMap(HashMap<String, Handle<Map<_Cell>>>);

	impl LoadAsset<Map<_Cell>> for _LoadMap {
		fn load_asset<'a, TPath: Into<AssetPath<'a>>>(&self, path: TPath) -> Handle<Map<_Cell>> {
			let path: AssetPath = path.into();
			self.0
				.iter()
				.find_map(|(key, value)| match AssetPath::from(key) == path {
					true => Some(value.clone()),
					false => None,
				})
				.unwrap_or(Handle::<Map<_Cell>>::Weak(AssetId::Uuid {
					uuid: Uuid::new_v4(),
				}))
				.clone()
		}
	}

	fn setup(load_map: _LoadMap) -> App {
		let mut app = App::new_single_threaded([Update]);
		app.insert_resource(load_map);
		app.add_systems(Update, begin_level_load::<_LoadMap, _Cell>);

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

		let level_command = app.world.get_resource::<LoadLevelCommand<_Cell>>();

		assert_eq!(Some(&LoadLevelCommand(handle)), level_command);
	}
}
