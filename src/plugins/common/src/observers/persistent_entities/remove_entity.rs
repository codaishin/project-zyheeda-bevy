use crate::{
	components::persistent_entity::PersistentEntity,
	resources::persistent_entities::PersistentEntities,
};
use bevy::prelude::*;

impl PersistentEntities {
	pub(crate) fn remove_entity(
		trigger: Trigger<OnRemove, PersistentEntity>,
		mut persistent_entities: ResMut<PersistentEntities>,
		entities: Query<&PersistentEntity>,
	) {
		let entity = trigger.target();
		let Ok(persistent_entity) = entities.get(entity) else {
			return;
		};

		persistent_entities.entities.remove(persistent_entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;
	use std::collections::HashMap;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<PersistentEntities>();
		app.add_observer(PersistentEntities::insert_entity);
		app.add_observer(PersistentEntities::remove_entity);

		app
	}

	#[test]
	fn remove() {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();
		let entity = app.world_mut().spawn(persistent_entity).id();

		app.world_mut().entity_mut(entity).despawn();

		assert!(
			app.world()
				.resource::<PersistentEntities>()
				.entities
				.is_empty()
		);
	}

	#[test]
	fn remove_matching() {
		let mut app = setup();
		let persistent_entity_a = PersistentEntity::default();
		let persistent_entity_b = PersistentEntity::default();
		let entity_a = app.world_mut().spawn(persistent_entity_a).id();
		let entity_b = app.world_mut().spawn(persistent_entity_b).id();

		app.world_mut().entity_mut(entity_a).despawn();

		assert_eq!(
			HashMap::from([(persistent_entity_b, entity_b)]),
			app.world().resource::<PersistentEntities>().entities,
		);
	}
}
