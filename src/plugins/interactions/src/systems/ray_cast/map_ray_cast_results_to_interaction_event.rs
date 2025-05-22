use super::execute_ray_caster::RayCastResult;
use crate::events::{Collision, InteractionEvent, Ray};
use bevy::prelude::*;
use common::components::collider_relationship::ColliderOfInteractionTarget;
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
		for (hit, ..) in &info.hits {
			let target_hit = get_target(*hit, &colliders);
			let event = event.collision(Collision::Started(target_hit));
			collisions.push(event);
		}

		events.push((ray, collisions));
	}

	events
}

fn get_target(entity: Entity, roots: &Query<&ColliderOfInteractionTarget>) -> Entity {
	match roots.get(entity) {
		Ok(root) => root.target(),
		Err(_) => entity,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::RayCastInfo;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::cast_ray::TimeOfImpact;

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
					hits: vec![
						(Entity::from_raw(42), TimeOfImpact(42.)),
						(Entity::from_raw(11), TimeOfImpact(11.)),
					],
					max_toi: TimeOfImpact(100.),
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
					TimeOfImpact(100.)
				),
				vec![
					interaction.collision(Collision::Started(Entity::from_raw(42))),
					interaction.collision(Collision::Started(Entity::from_raw(11))),
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
			.spawn(ColliderOfInteractionTarget::from_raw(target_a))
			.id();
		let collider_b = app
			.world_mut()
			.spawn(ColliderOfInteractionTarget::from_raw(target_b))
			.id();
		let ray_casts = HashMap::from([(
			Entity::from_raw(5),
			RayCastResult {
				info: RayCastInfo {
					hits: vec![
						(collider_a, TimeOfImpact(42.)),
						(collider_b, TimeOfImpact(11.)),
					],
					max_toi: TimeOfImpact(100.),
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
					TimeOfImpact(100.)
				),
				vec![
					interaction.collision(Collision::Started(target_a)),
					interaction.collision(Collision::Started(target_b)),
				]
			)],
			events
		);
		Ok(())
	}
}
