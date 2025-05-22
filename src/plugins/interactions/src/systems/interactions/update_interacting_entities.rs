use crate::{
	components::interacting_entities::InteractingEntities,
	events::{Collision, InteractionEvent},
};
use bevy::prelude::{EventReader, Query};

pub(crate) fn update_interacting_entities(
	mut events: EventReader<InteractionEvent>,
	mut agents: Query<&mut InteractingEntities>,
) {
	for InteractionEvent(a, collision) in events.read() {
		match collision {
			Collision::Started(b) => {
				if let Ok(mut agent) = agents.get_mut(a.0) {
					agent.0.insert(*b);
				}
				if let Ok(mut agent) = agents.get_mut(b.0) {
					agent.0.insert(*a);
				}
			}
			Collision::Ended(b) => {
				if let Ok(mut agent) = agents.get_mut(a.0) {
					agent.0.remove(b);
				}
				if let Ok(mut agent) = agents.get_mut(b.0) {
					agent.0.remove(a);
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::events::Collision;
	use bevy::{
		app::{App, Update},
		prelude::Entity,
	};
	use common::{components::collider_root::ColliderRoot, test_tools::utils::SingleThreadedApp};

	use super::*;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_event::<InteractionEvent>();
		app.add_systems(Update, update_interacting_entities);

		app
	}

	#[test]
	fn track_started_events() {
		let a = ColliderRoot(Entity::from_raw(9));
		let b = ColliderRoot(Entity::from_raw(10));
		let mut app = setup();
		let entity = app.world_mut().spawn(InteractingEntities::default()).id();

		app.world_mut().send_event_batch([
			InteractionEvent::of(a).collision(Collision::Started(ColliderRoot(entity))),
			InteractionEvent::of(ColliderRoot(entity)).collision(Collision::Started(b)),
		]);
		app.update();

		assert_eq!(
			Some(&InteractingEntities::new([a, b])),
			app.world().entity(entity).get::<InteractingEntities>()
		)
	}

	#[test]
	fn untrack_ended_events() {
		let a = ColliderRoot(Entity::from_raw(9));
		let b = ColliderRoot(Entity::from_raw(10));
		let c = ColliderRoot(Entity::from_raw(100));
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(InteractingEntities::new([a, b, c]))
			.id();

		app.world_mut().send_event_batch([
			InteractionEvent::of(a).collision(Collision::Ended(ColliderRoot(entity))),
			InteractionEvent::of(ColliderRoot(entity)).collision(Collision::Ended(b)),
		]);
		app.update();

		assert_eq!(
			Some(&InteractingEntities::new([c])),
			app.world().entity(entity).get::<InteractingEntities>()
		)
	}
}
