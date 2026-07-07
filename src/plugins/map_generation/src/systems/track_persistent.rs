use crate::components::{
	map::{Map, objects::MapObjects},
	spawned::Spawned,
};
use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

impl Map {
	pub(crate) fn track_persistent(
		maps: Query<(&mut Map, &MapObjects)>,
		spawned: Query<&Spawned, (With<PersistentEntity>, Added<Spawned>)>,
	) {
		for (mut map, objects) in maps {
			for obj in objects.iter() {
				let Ok(Spawned(obj_type)) = spawned.get(obj) else {
					continue;
				};

				map.persistent.insert(*obj_type);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		map::{MapObjectType, objects::MapObjectOf},
		spawned::Spawned,
	};
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::handles_map_generation::{AgentType, InteractiveType},
	};
	use std::collections::HashSet;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(Map::track_persistent, IsChanged::<Map>::detect).chain(),
		);

		app
	}

	#[test]
	fn set_persistent_map_agent_types() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Map::default()).id();
		app.world_mut().spawn((
			PersistentEntity::default(),
			Spawned(MapObjectType::Agent(AgentType::Player)),
			MapObjectOf(entity),
		));
		app.world_mut().spawn((
			PersistentEntity::default(),
			Spawned(MapObjectType::InteractiveType(InteractiveType::Container)),
			MapObjectOf(entity),
		));

		app.update();

		assert_eq!(
			Some(&Map {
				persistent: HashSet::from([
					MapObjectType::Agent(AgentType::Player),
					MapObjectType::InteractiveType(InteractiveType::Container)
				]),
			}),
			app.world().entity(entity).get::<Map>(),
		);
	}

	#[test]
	fn do_not_set_persistent_map_agent_types_without_persistent_entity() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Map::default()).id();
		app.world_mut().spawn((
			Spawned(MapObjectType::Agent(AgentType::Player)),
			MapObjectOf(entity),
		));
		app.world_mut().spawn((
			Spawned(MapObjectType::InteractiveType(InteractiveType::Container)),
			MapObjectOf(entity),
		));

		app.update();

		assert_eq!(
			Some(&Map {
				persistent: HashSet::from([]),
			}),
			app.world().entity(entity).get::<Map>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Map::default()).id();
		app.world_mut().spawn((
			PersistentEntity::default(),
			Spawned(MapObjectType::Agent(AgentType::Player)),
			MapObjectOf(entity),
		));
		app.world_mut().spawn((
			PersistentEntity::default(),
			Spawned(MapObjectType::InteractiveType(InteractiveType::Container)),
			MapObjectOf(entity),
		));

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<Map>>(),
		);
	}

	#[test]
	fn act_again_if_object_spawned() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Map::default()).id();
		app.world_mut().spawn((
			PersistentEntity::default(),
			Spawned(MapObjectType::Agent(AgentType::Player)),
			MapObjectOf(entity),
		));

		app.update();
		app.world_mut().spawn((
			PersistentEntity::default(),
			Spawned(MapObjectType::InteractiveType(InteractiveType::Container)),
			MapObjectOf(entity),
		));
		app.update();

		assert_eq!(
			Some(&Map {
				persistent: HashSet::from([
					MapObjectType::Agent(AgentType::Player),
					MapObjectType::InteractiveType(InteractiveType::Container)
				]),
			}),
			app.world().entity(entity).get::<Map>(),
		);
	}
}
