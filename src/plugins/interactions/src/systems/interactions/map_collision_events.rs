use crate::traits::{FromCollisionEvent, Track, TrackState};
use bevy::prelude::{Event, EventReader, EventWriter, Query, ResMut, Resource};
use bevy_rapier3d::prelude::CollisionEvent;
use common::components::ColliderRoot;

pub(crate) fn map_collision_events<TEvent, TEventTracker>(
	mut collisions: EventReader<CollisionEvent>,
	mut interactions: EventWriter<TEvent>,
	roots: Query<&ColliderRoot>,
	mut track: ResMut<TEventTracker>,
) where
	TEvent: Event + FromCollisionEvent,
	TEventTracker: Resource + Track<TEvent>,
{
	let get_root = |entity| match roots.get(entity) {
		Ok(root) => *root,
		Err(_) => ColliderRoot(entity),
	};

	for collision in collisions.read() {
		let event = TEvent::from_collision(collision, get_root);
		if track.track(&event) == TrackState::Changed {
			interactions.send(event);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::TrackState;
	use bevy::{
		app::{App, Update},
		prelude::{Entity, Events},
	};
	use bevy_rapier3d::rapier::prelude::CollisionEventFlags;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Resource, Default)]
	struct _Tracker<const STATE_CHANGED: bool>;

	impl<TEvent, const STATE_CHANGED: bool> Track<TEvent> for _Tracker<STATE_CHANGED> {
		fn track(&mut self, _: &TEvent) -> TrackState {
			if STATE_CHANGED {
				TrackState::Changed
			} else {
				TrackState::Unchanged
			}
		}
	}

	fn setup<TEvent: Event + FromCollisionEvent, const STATE_CHANGED: bool>() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Tracker<STATE_CHANGED>>();
		app.add_event::<CollisionEvent>();
		app.add_event::<TEvent>();
		app.add_systems(
			Update,
			map_collision_events::<TEvent, _Tracker<STATE_CHANGED>>,
		);

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

		let mut app = setup::<_Event, true>();

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

		let mut app = setup::<_Event, true>();

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

	#[test]
	fn do_not_send_event_when_tracker_state_unchanged() {
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

		let mut app = setup::<_Event, false>();

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

		assert_eq!(vec![] as Vec<&_Event>, events.collect::<Vec<_>>())
	}
}
