use crate::traits::FromCollisionEvent;
use bevy::prelude::{Event, EventReader, EventWriter, Query};
use bevy_rapier3d::prelude::CollisionEvent;
use common::components::ColliderRoot;

pub(crate) fn collision_event_to<TEvent>(
	mut collisions: EventReader<CollisionEvent>,
	mut interactions: EventWriter<TEvent>,
	roots: Query<&ColliderRoot>,
) where
	TEvent: Event + FromCollisionEvent,
{
	let get_root = |entity| match roots.get(entity) {
		Ok(root) => *root,
		Err(_) => ColliderRoot(entity),
	};

	for collision in collisions.read() {
		interactions.send(TEvent::from_collision(collision, get_root));
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

	fn setup<TEvent: Event + FromCollisionEvent>() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_event::<CollisionEvent>();
		app.add_event::<TEvent>();
		app.add_systems(Update, collision_event_to::<TEvent>);

		app
	}

	#[test]
	fn write_an_interaction_event_for_each_collision() {
		#[derive(Event, Debug, PartialEq)]
		struct _Event;

		impl FromCollisionEvent for _Event {
			fn from_collision<F>(_: &CollisionEvent, _: F) -> Self
			where
				F: Fn(Entity) -> ColliderRoot,
			{
				_Event
			}
		}

		let mut app = setup::<_Event>();

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

		let events = app.world().resource::<Events<_Event>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(vec![&_Event, &_Event], events.collect::<Vec<_>>())
	}

	#[test]
	fn write_an_interaction_event_for_each_collision_start_but_use_actual_collider_root_entities() {
		#[derive(Event, Debug, PartialEq)]
		struct _Event(ColliderRoot, ColliderRoot);

		impl FromCollisionEvent for _Event {
			fn from_collision<F>(c: &CollisionEvent, get_root: F) -> Self
			where
				F: Fn(Entity) -> ColliderRoot,
			{
				let (a, b) = match c {
					CollisionEvent::Started(a, b, ..) => (a, b),
					CollisionEvent::Stopped(a, b, ..) => (a, b),
				};
				_Event(get_root(*a), get_root(*b))
			}
		}

		let mut app = setup::<_Event>();

		let a = app.world_mut().spawn_empty().id();
		let collider_a = app.world_mut().spawn(ColliderRoot(a)).id();
		let b = app.world_mut().spawn_empty().id();
		let collider_b = app.world_mut().spawn(ColliderRoot(b)).id();

		app.world_mut().send_event(CollisionEvent::Started(
			collider_a,
			collider_b,
			CollisionEventFlags::empty(),
		));

		app.update();

		let events = app.world().resource::<Events<_Event>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(
			vec![&_Event(ColliderRoot(a), ColliderRoot(b))],
			events.collect::<Vec<_>>()
		)
	}
}
