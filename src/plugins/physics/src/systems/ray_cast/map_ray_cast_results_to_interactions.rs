use super::execute_ray_caster::RayCastResult;
use crate::{
	events::{Ray, RayEvent},
	traits::cast_ray::RayHit,
};
use bevy::prelude::*;
use std::collections::HashMap;

pub(crate) fn map_ray_cast_results_to_interactions(
	In(results): In<HashMap<Entity, RayCastResult>>,
	mut ray_events: EventWriter<RayEvent>,
) -> Vec<RayInteraction> {
	let mut intersections = vec![];

	for (ray_entity, RayCastResult { info }) in results.into_iter() {
		let mut last = None;

		for RayHit { entity, toi } in info.hits {
			intersections.push(RayInteraction {
				ray: ray_entity,
				intersecting: entity,
			});

			last = Some((info.ray, toi));
		}

		if let Some((ray, toi)) = last {
			ray_events.write(RayEvent(ray_entity, Ray(ray, toi)));
		};
	}

	intersections
}

#[derive(Debug, PartialEq)]
pub(crate) struct RayInteraction {
	pub(crate) ray: Entity,
	pub(crate) intersecting: Entity,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::{Ray, RayCastInfo};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::toi;
	use testing::{SingleThreadedApp, get_current_update_events};
	use zyheeda_core::prelude::Sorted;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_event::<RayEvent>();

		app
	}

	#[test]
	fn build_interactions() -> Result<(), RunSystemError> {
		let mut app = setup();

		let ray_casts = HashMap::from([(
			Entity::from_raw(5),
			RayCastResult {
				info: RayCastInfo {
					hits: Sorted::from([
						RayHit {
							entity: Entity::from_raw(42),
							toi: toi!(42.),
						},
						RayHit {
							entity: Entity::from_raw(11),
							toi: toi!(11.),
						},
					]),
					max_toi: toi!(100.),
					ray: Ray3d::new(
						Vec3::new(1., 2., 3.),
						Dir3::new_unchecked(Vec3::new(5., 6., 7.).normalize()),
					),
				},
			},
		)]);

		let intersections = app
			.world_mut()
			.run_system_once_with(map_ray_cast_results_to_interactions, ray_casts)?;

		assert_eq!(
			vec![
				RayInteraction {
					ray: Entity::from_raw(5),
					intersecting: Entity::from_raw(11)
				},
				RayInteraction {
					ray: Entity::from_raw(5),
					intersecting: Entity::from_raw(42)
				},
			],
			intersections
		);
		Ok(())
	}

	#[test]
	fn send_ray_event() -> Result<(), RunSystemError> {
		let mut app = setup();

		let ray_casts = HashMap::from([(
			Entity::from_raw(5),
			RayCastResult {
				info: RayCastInfo {
					hits: Sorted::from([
						RayHit {
							entity: Entity::from_raw(42),
							toi: toi!(42.),
						},
						RayHit {
							entity: Entity::from_raw(11),
							toi: toi!(11.),
						},
					]),
					max_toi: toi!(100.),
					ray: Ray3d::new(
						Vec3::new(1., 2., 3.),
						Dir3::new_unchecked(Vec3::new(5., 6., 7.).normalize()),
					),
				},
			},
		)]);

		app.world_mut()
			.run_system_once_with(map_ray_cast_results_to_interactions, ray_casts)?;

		assert_eq!(
			vec![&RayEvent(
				Entity::from_raw(5),
				Ray(
					Ray3d::new(
						Vec3::new(1., 2., 3.),
						Dir3::new_unchecked(Vec3::new(5., 6., 7.).normalize()),
					),
					toi!(42.)
				)
			)],
			get_current_update_events!(app, RayEvent).collect::<Vec<_>>()
		);
		Ok(())
	}
}
