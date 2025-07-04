use crate::{
	events::despawn_persistent::DespawnPersistent,
	resources::persistent_entities::PersistentEntities,
	traits::try_despawn::TryDespawn,
};
use bevy::prelude::*;

impl PersistentEntities {
	pub(crate) fn despawn_entity(
		trigger: Trigger<DespawnPersistent>,
		mut persistent_entities: ResMut<PersistentEntities>,
		mut commands: Commands,
	) {
		let DespawnPersistent(entity) = trigger.event();
		let Some(entity) = persistent_entities.get_entity(entity) else {
			return;
		};

		commands.try_despawn(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::persistent_entity::PersistentEntity;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<PersistentEntities>();
		app.add_observer(PersistentEntities::insert_entity);
		app.add_observer(PersistentEntities::despawn_entity);

		app
	}

	#[test]
	fn despawn() -> Result<(), RunSystemError> {
		let mut app = setup();
		let persistent = PersistentEntity::default();
		let entity = app.world_mut().spawn(persistent).id();

		app.world_mut()
			.run_system_once(move |mut commands: Commands| {
				commands.trigger(DespawnPersistent(persistent))
			})?;

		assert!(app.world().get_entity(entity).is_err());
		Ok(())
	}
}
