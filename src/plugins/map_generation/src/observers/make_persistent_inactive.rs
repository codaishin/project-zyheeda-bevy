use crate::components::{
	map::{Map, MapObjectType, objects::MapObjectOf},
	spawner::Spawner,
	spawner_active::SpawnerActive,
};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::TryApplyOn,
		handles_map_generation::PrefabType,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> Spawner<T>
where
	T: PrefabType + ThreadSafe + Copy + Into<MapObjectType>,
{
	pub(crate) fn make_persistent_inactive(
		on_add: On<Add, MapObjectOf>,
		mut commands: ZyheedaCommands,
		objects: Query<(&MapObjectOf, &Self)>,
		maps: Query<&Map>,
	) {
		let Ok((MapObjectOf(map), Self(obj_type))) = objects.get(on_add.entity) else {
			return;
		};
		let Ok(Map { persistent, .. }) = maps.get(*map) else {
			return;
		};
		let obj_type = (*obj_type).into();

		if !persistent.contains(&obj_type) {
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
	use common::traits::handles_map_generation::AgentType;
	use std::collections::HashSet;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(Spawner::<AgentType>::make_persistent_inactive);

		app
	}

	fn map_with_persistent<const N: usize>(persistent: [MapObjectType; N]) -> Map {
		Map {
			persistent: HashSet::from(persistent),
		}
	}

	#[test_case(map_with_persistent([MapObjectType::from(AgentType::Player)]), false; "spawned is persistent")]
	#[test_case(map_with_persistent([]), true; "spawned is not persistent")]
	fn control_active_marker(map: Map, is_active: bool) {
		let mut app = setup();
		let map = app.world_mut().spawn(map).id();

		let mut spawner = app.world_mut().spawn(Spawner(AgentType::Player));
		spawner.insert(MapObjectOf(map));

		assert_eq!(is_active, spawner.contains::<SpawnerActive>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let map = app
			.world_mut()
			.spawn(Map {
				persistent: HashSet::from([MapObjectType::from(AgentType::Player)]),
			})
			.id();

		let mut spawner = app.world_mut().spawn(Spawner(AgentType::Player));
		spawner.insert(MapObjectOf(map));
		spawner.insert(SpawnerActive);
		spawner.insert(MapObjectOf(map));

		assert!(spawner.contains::<SpawnerActive>());
	}
}
