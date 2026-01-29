use crate::{
	components::interaction_target::ColliderOfInteractionTarget,
	resources::ongoing_interactions::OngoingInteractions,
	traits::send_collision_interaction::PushOngoingInteraction,
};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub(crate) struct UpdateOngoingInteractions<'w, 's> {
	events: ResMut<'w, OngoingInteractions>,
	interaction_colliders: Query<'w, 's, &'static ColliderOfInteractionTarget>,
}

impl PushOngoingInteraction for UpdateOngoingInteractions<'_, '_> {
	fn push_ongoing_interaction(&mut self, actor: Entity, target: Entity) {
		let actor = self.get_root(actor);
		let target = self.get_root(target);
		let targets = self.events.targets.entry(actor).or_default();

		targets.insert(target);
	}
}

impl UpdateOngoingInteractions<'_, '_> {
	fn get_root(&self, entity: Entity) -> Entity {
		match self.interaction_colliders.get(entity) {
			Ok(ColliderOfInteractionTarget(target)) => *target,
			Err(_) => entity,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use std::collections::{HashMap, HashSet};
	use testing::{SingleThreadedApp, fake_entity};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<OngoingInteractions>();

		app
	}

	#[test]
	fn add_event_pair() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut sender: UpdateOngoingInteractions| {
				sender.push_ongoing_interaction(fake_entity!(1), fake_entity!(2));
			})?;

		assert_eq!(
			&OngoingInteractions {
				targets: HashMap::from([(fake_entity!(1), HashSet::from([fake_entity!(2)]))])
			},
			app.world().resource::<OngoingInteractions>()
		);
		Ok(())
	}

	#[test]
	fn add_entity_roots() -> Result<(), RunSystemError> {
		let mut app = setup();
		let roots = [
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
		];
		let colliders = [
			app.world_mut()
				.spawn(ColliderOfInteractionTarget(roots[0]))
				.id(),
			app.world_mut()
				.spawn(ColliderOfInteractionTarget(roots[1]))
				.id(),
		];

		app.world_mut()
			.run_system_once(move |mut sender: UpdateOngoingInteractions| {
				sender.push_ongoing_interaction(colliders[0], colliders[1]);
			})?;

		assert_eq!(
			&OngoingInteractions {
				targets: HashMap::from([(roots[0], HashSet::from([roots[1]]))])
			},
			app.world().resource::<OngoingInteractions>()
		);
		Ok(())
	}

	#[test]
	fn do_not_override_existing_entries() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut().insert_resource(OngoingInteractions {
			targets: HashMap::from([(fake_entity!(1), HashSet::from([fake_entity!(11)]))]),
		});
		app.world_mut()
			.run_system_once(move |mut sender: UpdateOngoingInteractions| {
				sender.push_ongoing_interaction(fake_entity!(1), fake_entity!(2));
			})?;

		assert_eq!(
			&OngoingInteractions {
				targets: HashMap::from([(
					fake_entity!(1),
					HashSet::from([fake_entity!(11), fake_entity!(2)])
				)])
			},
			app.world().resource::<OngoingInteractions>()
		);
		Ok(())
	}
}
