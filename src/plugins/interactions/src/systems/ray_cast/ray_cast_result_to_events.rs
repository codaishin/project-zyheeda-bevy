use crate::{
	components::RayCastResult,
	events::{InteractionEvent, Ray},
};
use bevy::prelude::{Commands, Entity, EventWriter, Query};
use common::traits::try_remove_from::TryRemoveFrom;

pub(crate) fn ray_cast_result_to_interaction_events(
	mut commands: Commands,
	results: Query<(Entity, &RayCastResult)>,
	mut interactions: EventWriter<InteractionEvent>,
	mut terminal_interactions: EventWriter<InteractionEvent<Ray>>,
) {
	for (entity, RayCastResult { info }) in &results {
		terminal_interactions.send(InteractionEvent::of(entity).ray(info.ray, info.max_toi));

		for (hit, ..) in &info.hits {
			interactions.send(InteractionEvent::of(entity).with(*hit));
		}

		commands.try_remove_from::<RayCastResult>(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::{Ray, RayCastInfo};
	use bevy::{
		app::{App, Update},
		math::{Dir3, Ray3d, Vec3},
		prelude::{default, Entity, Events},
	};
	use common::{test_tools::utils::SingleThreadedApp, traits::cast_ray::TimeOfImpact};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_event::<InteractionEvent>();
		app.add_event::<InteractionEvent<Ray>>();
		app.add_systems(Update, ray_cast_result_to_interaction_events);

		app
	}

	#[test]
	fn send_event_for_each_target_collision() {
		let mut app = setup();

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
				&InteractionEvent::of(ray_cast).with(Entity::from_raw(42)),
				&InteractionEvent::of(ray_cast).with(Entity::from_raw(11)),
			],
			events.collect::<Vec<_>>()
		);
	}

	#[test]
	fn send_ray_event_when_some_hits() {
		let mut app = setup();

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
			vec![&InteractionEvent::of(ray_cast).ray(ray, TimeOfImpact(999.))],
			events.collect::<Vec<_>>()
		);
	}

	#[test]
	fn send_ray_event_when_no_hits() {
		let mut app = setup();

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
			vec![&InteractionEvent::of(ray_cast).ray(ray, TimeOfImpact(567.))],
			events.collect::<Vec<_>>()
		);
	}

	#[test]
	fn remove_ray_cast_result() {
		let mut app = setup();

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
}
