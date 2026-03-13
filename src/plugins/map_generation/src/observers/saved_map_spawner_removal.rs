use crate::components::{
	agent_spawner::SpawnerActive,
	map::{Map, objects::MapObjectOf},
};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl SpawnerActive {
	pub(crate) fn remove_when_map_created_from_save(
		on_add: On<Add, MapObjectOf>,
		mut commands: ZyheedaCommands,
		objects: Query<&MapObjectOf>,
		maps: Query<&Map>,
	) {
		let Ok(MapObjectOf(map)) = objects.get(on_add.entity) else {
			return;
		};
		let Ok(Map { created_from_save }) = maps.get(*map) else {
			return;
		};

		if !*created_from_save {
			return;
		}

		commands.try_apply_on(&on_add.entity, |mut e| {
			e.try_remove::<SpawnerActive>();
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(SpawnerActive::remove_when_map_created_from_save);

		app
	}

	#[test_case(Map {created_from_save: true}, false; "map created from save")]
	#[test_case(Map {created_from_save: false}, true; "map not created from save")]
	fn control_active_marker(map: Map, is_active: bool) {
		let mut app = setup();
		let map = app.world_mut().spawn(map).id();

		let mut spawner = app.world_mut().spawn(SpawnerActive);
		spawner.insert(MapObjectOf(map));

		assert_eq!(is_active, spawner.contains::<SpawnerActive>(),);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let map = app
			.world_mut()
			.spawn(Map {
				created_from_save: true,
			})
			.id();

		let mut spawner = app.world_mut().spawn(SpawnerActive);
		spawner.insert(MapObjectOf(map));
		spawner.insert(SpawnerActive);
		spawner.insert(MapObjectOf(map));

		assert!(spawner.contains::<SpawnerActive>());
	}
}
