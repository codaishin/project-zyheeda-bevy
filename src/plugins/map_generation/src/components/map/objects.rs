use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::prelude::*;
use macros::{SavableComponent, serde_model};

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct MapObject;

#[derive(Component, Debug, PartialEq, Default)]
#[relationship_target(relationship = MapObjectOf, linked_spawn)]
pub(crate) struct MapObjects(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = MapObjects)]
pub(crate) struct MapObjectOf(pub(crate) Entity);

#[serde_model]
#[derive(SavableComponent, Component, Debug, PartialEq, Clone)]
#[savable_component(id = "map_object_of")]
pub(crate) struct PersistentMapObject {
	pub(crate) map: PersistentEntity,
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn despawn_object_on_map_despawn() {
		let mut app = setup();
		let map = app.world_mut().spawn_empty().id();
		let obj = app.world_mut().spawn(MapObjectOf(map)).id();

		app.world_mut().entity_mut(map).despawn();

		assert!(app.world().get_entity(obj).is_err());
	}
}
