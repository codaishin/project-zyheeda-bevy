use super::execute_ray_caster::RayCastResult;
use crate::{
	components::interaction_target::ColliderOfInteractionTarget,
	events::{Collision, InteractionEvent, Ray},
	traits::cast_ray::RayHit,
};
use bevy::prelude::*;
use std::collections::HashMap;

pub(crate) fn map_ray_cast_result_to_interaction_events(
	In(results): In<HashMap<Entity, RayCastResult>>,
	colliders: Query<&ColliderOfInteractionTarget>,
) -> Vec<(InteractionEvent<Ray>, Vec<InteractionEvent>)> {
	let mut events = vec![];

	for (entity, RayCastResult { info }) in results.into_iter() {
		let ray = InteractionEvent::of(entity).ray(info.ray, info.max_toi);
		let mut collisions = vec![];

		let target_entity = get_target(entity, &colliders);
		let event = InteractionEvent::of(target_entity);
		for RayHit { entity, .. } in info.hits {
			let target_hit = get_target(entity, &colliders);
			let event = event.collision(Collision::Started(target_hit));
			collisions.push(event);
		}

		events.push((ray, collisions));
	}

	events
}

fn get_target(entity: Entity, roots: &Query<&ColliderOfInteractionTarget>) -> Entity {
	match roots.get(entity) {
		Ok(ColliderOfInteractionTarget(root)) => *root,
		Err(_) => entity,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::RayCastInfo;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::toi;
	use zyheeda_core::prelude::Sorted;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn build_events() -> Result<(), RunSystemError> {
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

		let events = app
			.world_mut()
			.run_system_once_with(map_ray_cast_result_to_interaction_events, ray_casts)?;

		let interaction = InteractionEvent::of(Entity::from_raw(5));
		assert_eq!(
			vec![(
				interaction.ray(
					Ray3d::new(
						Vec3::new(1., 2., 3.),
						Dir3::new_unchecked(Vec3::new(5., 6., 7.).normalize())
					),
					toi!(100.)
				),
				vec![
					interaction.collision(Collision::Started(Entity::from_raw(11))),
					interaction.collision(Collision::Started(Entity::from_raw(42))),
				]
			)],
			events
		);
		Ok(())
	}

	#[test]
	fn send_event_for_each_target_collision_using_collider_root_reference()
	-> Result<(), RunSystemError> {
		let mut app = setup();

		let target_a = app.world_mut().spawn_empty().id();
		let target_b = app.world_mut().spawn_empty().id();
		let collider_a = app
			.world_mut()
			.spawn(ColliderOfInteractionTarget(target_a))
			.id();
		let collider_b = app
			.world_mut()
			.spawn(ColliderOfInteractionTarget(target_b))
			.id();
		let ray_casts = HashMap::from([(
			Entity::from_raw(5),
			RayCastResult {
				info: RayCastInfo {
					hits: Sorted::from([
						RayHit {
							entity: collider_a,
							toi: toi!(42.),
						},
						RayHit {
							entity: collider_b,
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

		let events = app
			.world_mut()
			.run_system_once_with(map_ray_cast_result_to_interaction_events, ray_casts)?;

		let interaction = InteractionEvent::of(Entity::from_raw(5));
		assert_eq!(
			vec![(
				interaction.ray(
					Ray3d::new(
						Vec3::new(1., 2., 3.),
						Dir3::new_unchecked(Vec3::new(5., 6., 7.).normalize())
					),
					toi!(100.)
				),
				vec![
					interaction.collision(Collision::Started(target_b)),
					interaction.collision(Collision::Started(target_a)),
				]
			)],
			events
		);
		Ok(())
	}
}
