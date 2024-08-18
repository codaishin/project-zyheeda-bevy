use bevy::prelude::{EventReader, EventWriter};
use bevy_rapier3d::prelude::CollisionEvent;

use crate::events::InteractionEvent;

pub(crate) fn collision_start_event_to_interaction_event(
	mut collisions: EventReader<CollisionEvent>,
	mut interactions: EventWriter<InteractionEvent>,
) {
	for collision in collisions.read() {
		let CollisionEvent::Started(a, b, ..) = collision else {
			continue;
		};
		interactions.send(InteractionEvent::of(*a).with(*b));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::{Entity, Events},
	};
	use bevy_rapier3d::rapier::prelude::CollisionEventFlags;
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_event::<CollisionEvent>();
		app.add_event::<InteractionEvent>();
		app.add_systems(Update, collision_start_event_to_interaction_event);

		app
	}

	#[test]
	fn write_an_interaction_event_for_each_collision_start() {
		let mut app = setup();

		app.world_mut().send_event(CollisionEvent::Started(
			Entity::from_raw(42),
			Entity::from_raw(90),
			CollisionEventFlags::empty(),
		));
		app.world_mut().send_event(CollisionEvent::Stopped(
			Entity::from_raw(9),
			Entity::from_raw(55),
			CollisionEventFlags::empty(),
		));

		app.update();

		let events = app.world().resource::<Events<InteractionEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(
			vec![&InteractionEvent::of(Entity::from_raw(42)).with(Entity::from_raw(90))],
			events.collect::<Vec<_>>()
		)
	}
}
