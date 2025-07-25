use crate::{
	events::{InteractionEvent, Ray},
	traits::{Flush, Track, TrackState},
};
use bevy::prelude::*;

pub(crate) fn send_interaction_events<TTracker>(
	In(interactions): In<Vec<(InteractionEvent<Ray>, Vec<InteractionEvent>)>>,
	mut tracker: ResMut<TTracker>,
	mut ray_events: EventWriter<InteractionEvent<Ray>>,
	mut interaction_events: EventWriter<InteractionEvent>,
) where
	TTracker: Resource + Track<InteractionEvent> + Flush<TResult = Vec<InteractionEvent>>,
{
	for (ray, collisions) in interactions.into_iter() {
		ray_events.write(ray);
		for collision in collisions.into_iter() {
			if tracker.track(&collision) == TrackState::Changed {
				interaction_events.write(collision);
			}
		}
	}
	interaction_events.write_batch(tracker.flush());
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::Collision;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::cast_ray::TimeOfImpact;
	use macros::NestedMocks;
	use mockall::{Sequence, mock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Tracker {
		mock: Mock_Tracker,
	}

	impl Flush for _Tracker {
		type TResult = Vec<InteractionEvent>;

		fn flush(&mut self) -> Self::TResult {
			self.mock.flush()
		}
	}

	impl Track<InteractionEvent> for _Tracker {
		fn track(&mut self, event: &InteractionEvent) -> TrackState {
			self.mock.track(event)
		}
	}

	mock! {
		_Tracker {}
		impl Flush for _Tracker {
			type TResult = Vec<InteractionEvent>;
			fn flush(&mut self) -> Vec<InteractionEvent>;
		}
		impl Track<InteractionEvent> for _Tracker {
			fn track(&mut self, event: &InteractionEvent) -> TrackState;
		}
	}

	fn setup(tracker: _Tracker) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(tracker);
		app.add_event::<InteractionEvent<Ray>>();
		app.add_event::<InteractionEvent>();

		app
	}

	#[test]
	fn call_flush() -> Result<(), RunSystemError> {
		let mut app = setup(_Tracker::new().with_mock(|mock| {
			mock.expect_track().return_const(TrackState::Changed);
			mock.expect_flush().times(1).return_const(vec![]);
		}));

		app.world_mut()
			.run_system_once_with(send_interaction_events::<_Tracker>, vec![])
	}

	#[test]
	fn send_flushed_events() -> Result<(), RunSystemError> {
		let a = Entity::from_raw(42);
		let b = Entity::from_raw(46);
		let mut app = setup(_Tracker::new().with_mock(|mock| {
			mock.expect_track().return_const(TrackState::Changed);
			mock.expect_flush().return_const(vec![
				InteractionEvent::of(a).collision(Collision::Started(b)),
			]);
		}));

		app.world_mut()
			.run_system_once_with(send_interaction_events::<_Tracker>, vec![])?;

		let events = app.world().resource::<Events<InteractionEvent>>();
		let mut cursor = events.get_cursor();
		let events = cursor.read(events);

		assert_eq!(
			vec![&InteractionEvent::of(a).collision(Collision::Started(b)),],
			events.collect::<Vec<_>>()
		);
		Ok(())
	}

	#[test]
	fn send_ray_events_from_input() -> Result<(), RunSystemError> {
		let interaction = InteractionEvent::of(Entity::from_raw(11));
		let ray = interaction.ray(
			Ray3d::new(
				Vec3::new(1., 2., 3.),
				Dir3::new_unchecked(Vec3::new(3., 2., 1.).normalize()),
			),
			TimeOfImpact(900.),
		);
		let collisions = vec![];
		let mut app = setup(_Tracker::new().with_mock(|mock| {
			mock.expect_track().return_const(TrackState::Changed);
			mock.expect_flush().return_const(vec![]);
		}));

		app.world_mut()
			.run_system_once_with(send_interaction_events::<_Tracker>, vec![(ray, collisions)])?;

		let events = app.world().resource::<Events<InteractionEvent<Ray>>>();
		let mut cursor = events.get_cursor();
		let events = cursor.read(events);

		assert_eq!(
			vec![&interaction.ray(
				Ray3d::new(
					Vec3::new(1., 2., 3.),
					Dir3::new_unchecked(Vec3::new(3., 2., 1.).normalize())
				),
				TimeOfImpact(900.),
			)],
			events.collect::<Vec<_>>()
		);
		Ok(())
	}

	#[test]
	fn send_changed_collider_events_from_input() -> Result<(), RunSystemError> {
		let interaction = InteractionEvent::of(Entity::from_raw(11));
		let ray = interaction.ray(Ray3d::new(default(), Dir3::X), default());
		let collisions = vec![
			interaction.collision(Collision::Started(Entity::from_raw(42))),
			interaction.collision(Collision::Started(Entity::from_raw(46))),
			interaction.collision(Collision::Started(Entity::from_raw(99))),
		];
		let mut app = setup(_Tracker::new().with_mock(|mock| {
			mock.expect_track()
				.with(eq(collisions[0]))
				.return_const(TrackState::Unchanged);
			mock.expect_track()
				.with(eq(collisions[1]))
				.return_const(TrackState::Changed);
			mock.expect_track()
				.with(eq(collisions[2]))
				.return_const(TrackState::Unchanged);
			mock.expect_flush().return_const(vec![]);
		}));

		app.world_mut()
			.run_system_once_with(send_interaction_events::<_Tracker>, vec![(ray, collisions)])?;

		let events = app.world().resource::<Events<InteractionEvent>>();
		let mut cursor = events.get_cursor();
		let events = cursor.read(events);

		assert_eq!(
			vec![&interaction.collision(Collision::Started(Entity::from_raw(46)))],
			events.collect::<Vec<_>>()
		);
		Ok(())
	}

	#[test]
	fn call_track_and_then_flush_in_correct_order() -> Result<(), RunSystemError> {
		let interaction = InteractionEvent::of(Entity::from_raw(11));
		let ray = interaction.ray(Ray3d::new(default(), Dir3::X), default());
		let collisions = vec![interaction.collision(Collision::Started(Entity::from_raw(42)))];
		let mut sequence = Sequence::new();
		let mut app = setup(_Tracker::new().with_mock(|mock| {
			mock.expect_track()
				.times(1)
				.in_sequence(&mut sequence)
				.return_const(TrackState::Changed);
			mock.expect_flush()
				.times(1)
				.in_sequence(&mut sequence)
				.return_const(vec![]);
		}));

		app.world_mut()
			.run_system_once_with(send_interaction_events::<_Tracker>, vec![(ray, collisions)])
	}
}
