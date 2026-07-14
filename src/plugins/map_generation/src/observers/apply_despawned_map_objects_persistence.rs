use crate::components::{
	map::{Map, objects::MapObjectOf},
	spawned_from::SpawnedFrom,
};
use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

impl Map {
	pub(crate) fn apply_despawned_map_objects_persistence(
		on_despawn: On<Despawn, SpawnedFrom>,
		mut maps: Query<&mut Self>,
		spawned: Query<(&SpawnedFrom, &MapObjectOf), With<PersistentEntity>>,
	) {
		let Ok((SpawnedFrom(source), MapObjectOf(map))) = spawned.get(on_despawn.entity) else {
			return;
		};

		let Ok(mut map) = maps.get_mut(*map) else {
			return;
		};

		map.disabled_object_sources.insert(source.clone());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::map::{MapObjectSource, objects::MapObjectOf};
	use common::components::persistent_entity::PersistentEntity;
	use std::collections::HashSet;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(Map::apply_despawned_map_objects_persistence);

		app
	}

	#[test]
	fn set_persistent_map_agent_types() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Map::default()).id();
		let entities = [
			app.world_mut()
				.spawn((
					PersistentEntity::default(),
					SpawnedFrom(MapObjectSource(String::from("a"))),
					MapObjectOf(entity),
				))
				.id(),
			app.world_mut()
				.spawn((
					PersistentEntity::default(),
					SpawnedFrom(MapObjectSource(String::from("b"))),
					MapObjectOf(entity),
				))
				.id(),
		];

		for entity in entities {
			app.world_mut().despawn(entity);
		}

		assert_eq!(
			Some(&Map {
				disabled_object_sources: HashSet::from([
					MapObjectSource(String::from("a")),
					MapObjectSource(String::from("b")),
				]),
			}),
			app.world().entity(entity).get::<Map>(),
		);
	}

	#[test]
	fn do_not_set_persistent_map_agent_types_when_not_persistent() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Map::default()).id();
		let entities = [
			app.world_mut()
				.spawn((
					SpawnedFrom(MapObjectSource(String::from("a"))),
					MapObjectOf(entity),
				))
				.id(),
			app.world_mut()
				.spawn((
					SpawnedFrom(MapObjectSource(String::from("b"))),
					MapObjectOf(entity),
				))
				.id(),
		];

		for entity in entities {
			app.world_mut().despawn(entity);
		}

		assert_eq!(
			Some(&Map {
				disabled_object_sources: HashSet::from([]),
			}),
			app.world().entity(entity).get::<Map>(),
		);
	}
}
