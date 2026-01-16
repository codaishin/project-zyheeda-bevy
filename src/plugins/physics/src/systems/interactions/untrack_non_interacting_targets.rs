use crate::components::{
	interacting_entities::InteractingEntities,
	running_interactions::RunningInteractions,
};
use bevy::prelude::*;
use common::{traits::accessors::get::Get, zyheeda_commands::ZyheedaCommands};

impl<TActor, TTarget> RunningInteractions<TActor, TTarget>
where
	TActor: Component,
	TTarget: Component,
{
	pub(crate) fn untrack_non_interacting_targets(
		mut agents: Query<(&mut Self, &InteractingEntities), Changed<InteractingEntities>>,
		commands: ZyheedaCommands,
	) {
		for (mut interactions, interacting_entities) in &mut agents {
			interactions.retain(|persistent_entity| {
				commands
					.get(persistent_entity)
					.map(|entity| interacting_entities.contains(&entity))
					.unwrap_or(false)
			});
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::register_persistent_entities::RegisterPersistentEntities,
	};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Actor;

	#[derive(Component)]
	struct _Target;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_persistent_entities();
		app.add_systems(
			Update,
			RunningInteractions::<_Actor, _Target>::untrack_non_interacting_targets,
		);

		app
	}

	#[test]
	fn remove_entities_not_contained_in_interacting_entities() {
		let mut app = setup();
		let not_interacting = PersistentEntity::default();
		let interacting = [PersistentEntity::default(), PersistentEntity::default()];
		let interacting_entities = interacting.map(|e| app.world_mut().spawn(e).id());
		let entity = app
			.world_mut()
			.spawn((
				InteractingEntities::new(interacting_entities),
				RunningInteractions::<_Actor, _Target>::from([
					interacting[0],
					interacting[1],
					not_interacting,
				]),
			))
			.id();

		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(&RunningInteractions::<_Actor, _Target>::from(interacting)),
			entity.get::<RunningInteractions<_Actor, _Target>>(),
		)
	}

	#[test]
	fn do_not_remove_entity_when_interacting_entities_not_changed() {
		let mut app = setup();
		let interacting = [PersistentEntity::default(), PersistentEntity::default()];
		let interacting_entities = interacting.map(|e| app.world_mut().spawn(e).id());
		let entity = app
			.world_mut()
			.spawn((
				InteractingEntities::new(interacting_entities),
				RunningInteractions::<_Actor, _Target>::from(interacting),
			))
			.id();

		app.update();
		let extra = PersistentEntity::default();
		app.world_mut().spawn(extra);
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<RunningInteractions<_Actor, _Target>>()
			.unwrap()
			.insert(extra);
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(&RunningInteractions::<_Actor, _Target>::from([
				interacting[0],
				interacting[1],
				extra,
			])),
			entity.get::<RunningInteractions<_Actor, _Target>>(),
		)
	}

	#[test]
	fn remove_entity_when_interacting_entity_changed() {
		let mut app = setup();
		let interacting = [PersistentEntity::default(), PersistentEntity::default()];
		let interacting_entities = interacting.map(|e| app.world_mut().spawn(e).id());
		let entity = app
			.world_mut()
			.spawn((
				InteractingEntities::new(interacting_entities),
				RunningInteractions::<_Actor, _Target>::from(interacting),
			))
			.id();

		app.update();
		let extra = PersistentEntity::default();
		app.world_mut().spawn(extra);
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<RunningInteractions<_Actor, _Target>>()
			.unwrap()
			.insert(extra);
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<InteractingEntities>()
			.as_deref_mut();
		app.update();

		let entity = app.world().entity(entity);
		assert_eq!(
			Some(&RunningInteractions::<_Actor, _Target>::from(interacting)),
			entity.get::<RunningInteractions<_Actor, _Target>>(),
		)
	}
}
