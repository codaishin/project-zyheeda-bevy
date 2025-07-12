use super::TryDespawn;
use crate::{
	components::persistent_entity::PersistentEntity,
	events::despawn_persistent::DespawnPersistent,
	traits::try_despawn::TryDespawnPersistent,
};
use bevy::prelude::*;

impl TryDespawn for Commands<'_, '_> {
	fn try_despawn(&mut self, entity: Entity) {
		let Ok(mut entity) = self.get_entity(entity) else {
			return;
		};
		entity.try_despawn();
	}
}

impl TryDespawnPersistent for Commands<'_, '_> {
	fn try_despawn_persistent(&mut self, entity: PersistentEntity) {
		self.trigger(DespawnPersistent(entity));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn despawn_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut commands: Commands| commands.try_despawn(entity))?;

		assert!(app.world().get_entity(entity).is_err());
		Ok(())
	}

	#[test]
	fn despawn_entity_children() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn(ChildOf(entity)).id();

		app.world_mut()
			.run_system_once(move |mut commands: Commands| commands.try_despawn(entity))?;

		assert!(app.world().get_entity(child).is_err());
		Ok(())
	}

	#[test]
	fn no_panic_when_entity_does_not_exist() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = Entity::from_raw(1000);

		app.world_mut()
			.run_system_once(move |mut commands: Commands| commands.try_despawn(entity))
	}

	#[test]
	fn trigger_despawn_persistent() -> Result<(), RunSystemError> {
		#[derive(Resource, Debug, PartialEq)]
		struct _Event(DespawnPersistent);

		let mut app = setup();
		let persistent = PersistentEntity::default();

		app.add_observer(
			move |trigger: Trigger<DespawnPersistent>, mut commands: Commands| {
				commands.insert_resource(_Event(*trigger.event()))
			},
		);
		app.world_mut()
			.run_system_once(move |mut commands: Commands| {
				commands.try_despawn_persistent(persistent)
			})?;

		assert_eq!(
			Some(&_Event(DespawnPersistent(persistent))),
			app.world().get_resource::<_Event>()
		);
		Ok(())
	}
}
