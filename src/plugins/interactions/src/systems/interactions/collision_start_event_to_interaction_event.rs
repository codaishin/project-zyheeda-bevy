use crate::events::{Collision, InteractionEvent};
use bevy::prelude::{Entity, EventReader, EventWriter, Query};
use bevy_rapier3d::prelude::CollisionEvent;
use common::components::ColliderRoot;

pub(crate) fn collision_start_event_to_interaction_event(
	mut collisions: EventReader<CollisionEvent>,
	mut interactions: EventWriter<InteractionEvent>,
	roots: Query<&ColliderRoot>,
) {
	let roots = &roots;

	for collision in collisions.read() {
		let CollisionEvent::Started(a, b, ..) = collision else {
			continue;
		};
		let a = get_root(*a, roots);
		let b = get_root(*b, roots);
		interactions.send(InteractionEvent::of(a).collision(Collision::Started(b)));
	}
}

fn get_root(entity: Entity, roots: &Query<&ColliderRoot>) -> ColliderRoot {
	match roots.get(entity) {
		Ok(root) => *root,
		Err(_) => ColliderRoot(entity),
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
			vec![&InteractionEvent::of(ColliderRoot(Entity::from_raw(42)))
				.collision(Collision::Started(ColliderRoot(Entity::from_raw(90))))],
			events.collect::<Vec<_>>()
		)
	}

	#[test]
	fn write_an_interaction_event_for_each_collision_start_but_use_actual_collider_root_entities() {
		let mut app = setup();

		let a = app.world_mut().spawn_empty().id();
		let collider_a = app.world_mut().spawn(ColliderRoot(a)).id();
		let b = app.world_mut().spawn_empty().id();
		let collider_b = app.world_mut().spawn(ColliderRoot(b)).id();

		app.world_mut().send_event(CollisionEvent::Started(
			collider_a,
			collider_b,
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
			vec![&InteractionEvent::of(ColliderRoot(a)).collision(Collision::Started(ColliderRoot(b)))],
			events.collect::<Vec<_>>()
		)
	}
}
