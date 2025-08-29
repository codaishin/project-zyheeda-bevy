use crate::{
	components::persistent_entity::PersistentEntity,
	resources::persistent_entities::PersistentEntities,
};
use bevy::prelude::*;

impl PersistentEntities {
	pub(crate) fn insert_entity(
		trigger: Trigger<OnInsert, PersistentEntity>,
		persistent_entities: Option<ResMut<Self>>,
		entities: Query<&PersistentEntity>,
	) {
		let entity = trigger.target();
		let Ok(persistent_entity) = entities.get(entity) else {
			return;
		};

		let Some(mut persistent_entities) = persistent_entities else {
			return;
		};

		persistent_entities
			.entities
			.insert(*persistent_entity, entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<PersistentEntities>();
		app.add_observer(PersistentEntities::insert_entity);

		app
	}

	#[test]
	fn update_with_entity() {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();

		let entity = app.world_mut().spawn(persistent_entity).id();

		assert_eq!(
			Some(&entity),
			app.world()
				.resource::<PersistentEntities>()
				.entities
				.get(&persistent_entity)
		);
	}

	#[test]
	fn update_with_entity_when_reinserted() {
		let mut app = setup();
		let persistent_entity = PersistentEntity::default();

		let mut entity = app.world_mut().spawn(PersistentEntity::default());
		entity.insert(persistent_entity);

		let entity = entity.id();
		assert_eq!(
			Some(&entity),
			app.world()
				.resource::<PersistentEntities>()
				.entities
				.get(&persistent_entity)
		);
	}
}
