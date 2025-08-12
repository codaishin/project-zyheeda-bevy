pub(crate) mod insert_entity;
pub(crate) mod remove_entity;

#[cfg(test)]
mod integration_tests {
	use crate::{
		components::persistent_entity::PersistentEntity,
		resources::persistent_entities::PersistentEntities,
		traits::{
			accessors::get::{GetMut, TryApplyOn},
			register_persistent_entities::RegisterPersistentEntities,
		},
		zyheeda_commands::ZyheedaCommands,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();

		app
	}

	#[test]
	fn no_mem_leak_on_entity_despawn() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(PersistentEntity::default()).id();

		app.world_mut()
			.run_system_once(move |mut commands: ZyheedaCommands| {
				commands.try_apply_on(&entity, |e| e.try_despawn());
			})?;

		assert!(
			app.world()
				.resource::<PersistentEntities>()
				.entities
				.is_empty()
		);
		Ok(())
	}

	#[test]
	fn no_mem_leak_on_persistent_entity_despawn() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = PersistentEntity::default();
		app.world_mut().spawn(entity);

		app.world_mut()
			.run_system_once(move |mut commands: ZyheedaCommands| {
				let Some(entity) = commands.get_mut(&entity) else {
					return;
				};
				entity.try_despawn();
			})?;

		assert!(
			app.world()
				.resource::<PersistentEntities>()
				.entities
				.is_empty()
		);
		Ok(())
	}
}
