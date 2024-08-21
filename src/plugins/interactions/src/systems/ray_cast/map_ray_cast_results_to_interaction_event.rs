use crate::{
	components::RayCastResult,
	events::{Collision, InteractionEvent, Ray},
	traits::{Track, TrackState},
};
use bevy::prelude::{Commands, Entity, EventWriter, Query, ResMut, Resource};
use common::{components::ColliderRoot, traits::try_remove_from::TryRemoveFrom};

pub(crate) fn map_ray_cast_result_to_interaction_changes<TTracker>(
	mut commands: Commands,
	results: Query<(Entity, &RayCastResult)>,
	mut interactions: EventWriter<InteractionEvent>,
	mut terminal_interactions: EventWriter<InteractionEvent<Ray>>,
	roots: Query<&ColliderRoot>,
	mut tracker: ResMut<TTracker>,
) where
	TTracker: Resource + Track<InteractionEvent>,
{
	let roots = &roots;

	for (entity, RayCastResult { info }) in &results {
		terminal_interactions
			.send(InteractionEvent::of(ColliderRoot(entity)).ray(info.ray, info.max_toi));

		let root_entity = get_root(entity, roots);
		for (hit, ..) in &info.hits {
			let root_hit = get_root(*hit, roots);
			let event = InteractionEvent::of(root_entity).collision(Collision::Started(root_hit));

			if tracker.track(&event) == TrackState::Unchanged {
				continue;
			}

			interactions.send(event);
		}

		commands.try_remove_from::<RayCastResult>(entity);
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
	use crate::{
		events::{Ray, RayCastInfo},
		traits::TrackState,
	};
	use bevy::{
		app::{App, Update},
		math::{Dir3, Ray3d, Vec3},
		prelude::{default, Entity, Events},
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{cast_ray::TimeOfImpact, nested_mock::NestedMock},
	};
	use macros::NestedMock;
	use mockall::{automock, predicate::eq};

	#[derive(Resource, NestedMock)]
	struct _Tracker {
		mock: Mock_Tracker,
	}

	#[automock]
	impl Track<InteractionEvent> for _Tracker {
		fn track(&mut self, event: &InteractionEvent) -> crate::traits::TrackState {
			self.mock.track(event)
		}
	}

	fn setup(tracker: _Tracker) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_event::<InteractionEvent>();
		app.add_event::<InteractionEvent<Ray>>();
		app.insert_resource(tracker);
		app.add_systems(
			Update,
			map_ray_cast_result_to_interaction_changes::<_Tracker>,
		);

		app
	}

	#[test]
	fn send_event_for_each_target_collision() {
		let mut app = setup(_Tracker::new_mock(|mock| {
			mock.expect_track().return_const(TrackState::Changed);
		}));

		let ray_cast = app
			.world_mut()
			.spawn(RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(Entity::from_raw(42), TimeOfImpact(42.)),
						(Entity::from_raw(11), TimeOfImpact(11.)),
					],
					..default()
				},
			})
			.id();

		app.update();

		let events = app.world().resource::<Events<InteractionEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(
			vec![
				&InteractionEvent::of(ColliderRoot(ray_cast))
					.collision(Collision::Started(ColliderRoot(Entity::from_raw(42)))),
				&InteractionEvent::of(ColliderRoot(ray_cast))
					.collision(Collision::Started(ColliderRoot(Entity::from_raw(11)))),
			],
			events.collect::<Vec<_>>()
		);
	}

	#[test]
	fn send_event_for_each_target_collision_using_collider_root_reference() {
		let mut app = setup(_Tracker::new_mock(|mock| {
			mock.expect_track().return_const(TrackState::Changed);
		}));

		let collider_a = app
			.world_mut()
			.spawn(ColliderRoot(Entity::from_raw(42)))
			.id();
		let collider_b = app
			.world_mut()
			.spawn(ColliderRoot(Entity::from_raw(11)))
			.id();
		let ray_cast = app
			.world_mut()
			.spawn(RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(collider_a, TimeOfImpact(42.)),
						(collider_b, TimeOfImpact(11.)),
					],
					..default()
				},
			})
			.id();

		app.update();

		let events = app.world().resource::<Events<InteractionEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(
			vec![
				&InteractionEvent::of(ColliderRoot(ray_cast))
					.collision(Collision::Started(ColliderRoot(Entity::from_raw(42)))),
				&InteractionEvent::of(ColliderRoot(ray_cast))
					.collision(Collision::Started(ColliderRoot(Entity::from_raw(11)))),
			],
			events.collect::<Vec<_>>()
		);
	}

	#[test]
	fn send_ray_event_when_some_hits() {
		let mut app = setup(_Tracker::new_mock(|mock| {
			mock.expect_track().return_const(TrackState::Changed);
		}));

		let ray = Ray3d {
			origin: Vec3::Z,
			direction: Dir3::Y,
		};
		let ray_cast = app
			.world_mut()
			.spawn(RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(Entity::from_raw(42), TimeOfImpact(42.)),
						(Entity::from_raw(11), TimeOfImpact(11.)),
					],
					ray,
					max_toi: TimeOfImpact(999.),
				},
			})
			.id();

		app.update();

		let events = app.world().resource::<Events<InteractionEvent<Ray>>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(
			vec![&InteractionEvent::of(ColliderRoot(ray_cast)).ray(ray, TimeOfImpact(999.))],
			events.collect::<Vec<_>>()
		);
	}

	#[test]
	fn send_ray_event_when_no_hits() {
		let mut app = setup(_Tracker::new_mock(|mock| {
			mock.expect_track().return_const(TrackState::Changed);
		}));

		let ray = Ray3d {
			origin: Vec3::Z,
			direction: Dir3::Y,
		};
		let ray_cast = app
			.world_mut()
			.spawn(RayCastResult {
				info: RayCastInfo {
					hits: vec![],
					max_toi: TimeOfImpact(567.),
					ray,
				},
			})
			.id();

		app.update();

		let events = app.world().resource::<Events<InteractionEvent<Ray>>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(
			vec![&InteractionEvent::of(ColliderRoot(ray_cast)).ray(ray, TimeOfImpact(567.))],
			events.collect::<Vec<_>>()
		);
	}

	#[test]
	fn remove_ray_cast_result() {
		let mut app = setup(_Tracker::new_mock(|mock| {
			mock.expect_track().return_const(TrackState::Changed);
		}));

		let ray_cast = app
			.world_mut()
			.spawn(RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(Entity::from_raw(42), TimeOfImpact(42.)),
						(Entity::from_raw(11), TimeOfImpact(11.)),
					],
					..default()
				},
			})
			.id();

		app.update();

		let ray_cast = app.world().entity(ray_cast);

		assert_eq!(None, ray_cast.get::<RayCastResult>());
	}

	#[test]
	fn do_sent_interaction_collision_event_when_tracker_unchanged() {
		let mut app = setup(_Tracker::new_mock(|mock| {
			mock.expect_track().return_const(TrackState::Unchanged);
		}));

		app.world_mut().spawn(RayCastResult {
			info: RayCastInfo {
				hits: vec![
					(Entity::from_raw(42), TimeOfImpact(42.)),
					(Entity::from_raw(11), TimeOfImpact(11.)),
				],
				..default()
			},
		});

		app.update();

		let events = app.world().resource::<Events<InteractionEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(vec![] as Vec<&InteractionEvent>, events.collect::<Vec<_>>());
	}

	#[test]
	fn call_track_with_proper_interaction() {
		let mut app = setup(_Tracker::new_mock(|_| {}));

		let ray_cast = app
			.world_mut()
			.spawn(RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(Entity::from_raw(42), TimeOfImpact(42.)),
						(Entity::from_raw(11), TimeOfImpact(11.)),
					],
					..default()
				},
			})
			.id();

		app.insert_resource(_Tracker::new_mock(|mock| {
			mock.expect_track()
				.times(1)
				.with(eq(InteractionEvent::of(ColliderRoot(ray_cast)).collision(
					Collision::Started(ColliderRoot(Entity::from_raw(42))),
				)))
				.return_const(TrackState::Changed);
			mock.expect_track()
				.times(1)
				.with(eq(InteractionEvent::of(ColliderRoot(ray_cast)).collision(
					Collision::Started(ColliderRoot(Entity::from_raw(11))),
				)))
				.return_const(TrackState::Unchanged);
		}));
		app.update();

		let events = app.world().resource::<Events<InteractionEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(
			vec![&InteractionEvent::of(ColliderRoot(ray_cast))
				.collision(Collision::Started(ColliderRoot(Entity::from_raw(42))))],
			events.collect::<Vec<_>>()
		);
	}
}
