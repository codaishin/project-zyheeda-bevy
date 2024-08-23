use crate::{
	events::{InteractionEvent, Ray},
	traits::{Flush, Track, TrackState},
};
use bevy::prelude::{EventWriter, In, ResMut, Resource};

pub(crate) fn send_interaction_events<TTracker>(
	In(interactions): In<Vec<(InteractionEvent<Ray>, Vec<InteractionEvent>)>>,
	mut tracker: ResMut<TTracker>,
	mut ray_events: EventWriter<InteractionEvent<Ray>>,
	mut interaction_events: EventWriter<InteractionEvent>,
) where
	TTracker: Resource + Track<InteractionEvent> + Flush<TResult = Vec<InteractionEvent>>,
{
	for (ray, collisions) in interactions.into_iter() {
		ray_events.send(ray);
		for collision in collisions.into_iter() {
			if tracker.track(&collision) == TrackState::Changed {
				interaction_events.send(collision);
			}
		}
	}
	interaction_events.send_batch(tracker.flush());
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::Collision;
	use bevy::{
		app::{App, Update},
		ecs::system::RunSystemOnce,
		math::{Ray3d, Vec3},
		prelude::{Entity, Events},
		utils::default,
	};
	use common::{
		components::ColliderRoot,
		test_tools::utils::SingleThreadedApp,
		traits::{cast_ray::TimeOfImpact, nested_mock::NestedMock},
	};
	use macros::NestedMock;
	use mockall::{mock, predicate::eq, Sequence};

	#[derive(Resource, NestedMock)]
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
	fn call_flush() {
		let mut app = setup(_Tracker::new_mock(|mock| {
			mock.expect_track().return_const(TrackState::Changed);
			mock.expect_flush().times(1).return_const(vec![]);
		}));

		app.world_mut()
			.run_system_once_with(vec![], send_interaction_events::<_Tracker>);
	}

	#[test]
	fn send_flushed_events() {
		let a = ColliderRoot(Entity::from_raw(42));
		let b = ColliderRoot(Entity::from_raw(46));
		let mut app = setup(_Tracker::new_mock(|mock| {
			mock.expect_track().return_const(TrackState::Changed);
			mock.expect_flush().return_const(vec![
				InteractionEvent::of(a).collision(Collision::Started(b))
			]);
		}));

		app.world_mut()
			.run_system_once_with(vec![], send_interaction_events::<_Tracker>);

		let events = app.world().resource::<Events<InteractionEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(
			vec![&InteractionEvent::of(a).collision(Collision::Started(b)),],
			events.collect::<Vec<_>>()
		)
	}

	#[test]
	fn send_ray_events_from_input() {
		let interaction = InteractionEvent::of(ColliderRoot(Entity::from_raw(11)));
		let ray = interaction.ray(
			Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(3., 2., 1.)),
			TimeOfImpact(900.),
		);
		let collisions = vec![];
		let mut app = setup(_Tracker::new_mock(|mock| {
			mock.expect_track().return_const(TrackState::Changed);
			mock.expect_flush().return_const(vec![]);
		}));

		app.world_mut()
			.run_system_once_with(vec![(ray, collisions)], send_interaction_events::<_Tracker>);

		let events = app.world().resource::<Events<InteractionEvent<Ray>>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(
			vec![&interaction.ray(
				Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(3., 2., 1.)),
				TimeOfImpact(900.),
			)],
			events.collect::<Vec<_>>()
		)
	}

	#[test]
	fn send_changed_collider_events_from_input() {
		let interaction = InteractionEvent::of(ColliderRoot(Entity::from_raw(11)));
		let ray = interaction.ray(Ray3d::new(default(), Vec3::X), default());
		let collisions = vec![
			interaction.collision(Collision::Started(ColliderRoot(Entity::from_raw(42)))),
			interaction.collision(Collision::Started(ColliderRoot(Entity::from_raw(46)))),
			interaction.collision(Collision::Started(ColliderRoot(Entity::from_raw(99)))),
		];
		let mut app = setup(_Tracker::new_mock(|mock| {
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
			.run_system_once_with(vec![(ray, collisions)], send_interaction_events::<_Tracker>);

		let events = app.world().resource::<Events<InteractionEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(
			vec![&interaction.collision(Collision::Started(ColliderRoot(Entity::from_raw(46))))],
			events.collect::<Vec<_>>()
		)
	}

	#[test]
	fn call_track_and_then_flush_in_correct_order() {
		let interaction = InteractionEvent::of(ColliderRoot(Entity::from_raw(11)));
		let ray = interaction.ray(Ray3d::new(default(), Vec3::X), default());
		let collisions =
			vec![interaction.collision(Collision::Started(ColliderRoot(Entity::from_raw(42))))];
		let mut sequence = Sequence::new();
		let mut app = setup(_Tracker::new_mock(|mock| {
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
			.run_system_once_with(vec![(ray, collisions)], send_interaction_events::<_Tracker>);
	}
}
