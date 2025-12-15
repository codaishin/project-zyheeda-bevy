use crate::{
	components::interaction_target::ColliderOfInteractionTarget,
	traits::{FromCollisionEvent, Track, TrackState},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::CollisionEvent;

pub(crate) fn map_collision_events_to<TEvent, TEventTracker>(
	mut collisions: EventReader<CollisionEvent>,
	mut interactions: EventWriter<TEvent>,
	colliders: Query<&ColliderOfInteractionTarget>,
	mut track: ResMut<TEventTracker>,
) where
	TEvent: Event + FromCollisionEvent,
	TEventTracker: Resource + Track<TEvent>,
{
	let get_target = |entity| match colliders.get(entity) {
		Ok(ColliderOfInteractionTarget(target)) => *target,
		Err(_) => entity,
	};

	for collision in collisions.read() {
		let event = TEvent::from_collision(collision, get_target);
		if track.track(&event) == TrackState::Changed {
			interactions.write(event);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::TrackState;
	use bevy_rapier3d::rapier::prelude::CollisionEventFlags;
	use testing::{SingleThreadedApp, get_current_update_events};

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
			map_collision_events_to::<TEvent, _Tracker<STATE_CHANGED>>,
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
				F: Fn(Entity) -> Entity,
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

		assert_eq!(
			vec![&_Event, &_Event],
			get_current_update_events!(app, _Event).collect::<Vec<_>>()
		)
	}

	#[test]
	fn write_an_interaction_event_for_each_collision_start_but_use_actual_collider_root_entities() {
		#[derive(Event, Debug, PartialEq)]
		struct _Event(Entity, Entity);

		impl FromCollisionEvent for _Event {
			fn from_collision<F>(c: &CollisionEvent, get_root: F) -> Self
			where
				F: Fn(Entity) -> Entity,
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
		let collider_a = app.world_mut().spawn(ColliderOfInteractionTarget(a)).id();
		let b = app.world_mut().spawn_empty().id();
		let collider_b = app.world_mut().spawn(ColliderOfInteractionTarget(b)).id();

		app.world_mut().send_event(CollisionEvent::Started(
			collider_a,
			collider_b,
			CollisionEventFlags::empty(),
		));

		app.update();

		assert_eq!(
			vec![&_Event(a, b)],
			get_current_update_events!(app, _Event).collect::<Vec<_>>()
		)
	}

	#[test]
	fn do_not_send_event_when_tracker_state_unchanged() {
		#[derive(Event, Debug, PartialEq)]
		struct _Event;

		impl FromCollisionEvent for _Event {
			fn from_collision<F>(_: &CollisionEvent, _: F) -> Self
			where
				F: Fn(Entity) -> Entity,
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

		assert_eq!(
			vec![] as Vec<&_Event>,
			get_current_update_events!(app, _Event).collect::<Vec<_>>()
		)
	}
}
