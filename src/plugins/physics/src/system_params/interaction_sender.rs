use crate::{
	components::interaction_target::ColliderOfInteractionTarget,
	events::{Collision, InteractionEvent},
	traits::send_collision_interaction::SendCollisionInteraction,
};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub(crate) struct InteractionSender<'w, 's> {
	event_writer: EventWriter<'w, InteractionEvent>,
	interaction_colliders: Query<'w, 's, &'static ColliderOfInteractionTarget>,
}

impl SendCollisionInteraction for InteractionSender<'_, '_> {
	fn start_interaction(&mut self, a: Entity, b: Entity) {
		self.event_writer.write(
			InteractionEvent::of(self.get_root(a)).collision(Collision::Started(self.get_root(b))),
		);
	}

	fn end_interaction(&mut self, a: Entity, b: Entity) {
		self.event_writer.write(
			InteractionEvent::of(self.get_root(a)).collision(Collision::Ended(self.get_root(b))),
		);
	}
}

impl InteractionSender<'_, '_> {
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
	use testing::{SingleThreadedApp, get_current_update_events};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_event::<InteractionEvent>();

		app
	}

	#[test]
	fn send_events() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut sender: InteractionSender| {
				sender.start_interaction(Entity::from_raw(1), Entity::from_raw(2));
				sender.end_interaction(Entity::from_raw(3), Entity::from_raw(4));
			})?;

		assert_eq!(
			vec![
				&InteractionEvent::of(Entity::from_raw(1))
					.collision(Collision::Started(Entity::from_raw(2))),
				&InteractionEvent::of(Entity::from_raw(3))
					.collision(Collision::Ended(Entity::from_raw(4))),
			],
			get_current_update_events!(app, InteractionEvent).collect::<Vec<_>>(),
		);
		Ok(())
	}

	#[test]
	fn send_events_through_collider_targets() -> Result<(), RunSystemError> {
		let mut app = setup();
		let roots = [
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
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
			app.world_mut()
				.spawn(ColliderOfInteractionTarget(roots[2]))
				.id(),
			app.world_mut()
				.spawn(ColliderOfInteractionTarget(roots[3]))
				.id(),
		];

		app.world_mut()
			.run_system_once(move |mut sender: InteractionSender| {
				sender.start_interaction(colliders[0], colliders[1]);
				sender.end_interaction(colliders[2], colliders[3]);
			})?;

		assert_eq!(
			vec![
				&InteractionEvent::of(roots[0]).collision(Collision::Started(roots[1])),
				&InteractionEvent::of(roots[2]).collision(Collision::Ended(roots[3])),
			],
			get_current_update_events!(app, InteractionEvent).collect::<Vec<_>>(),
		);
		Ok(())
	}
}
