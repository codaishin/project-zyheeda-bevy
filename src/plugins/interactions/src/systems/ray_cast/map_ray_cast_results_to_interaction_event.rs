use super::execute_ray_caster::RayCastResult;
use crate::events::{Collision, InteractionEvent, Ray};
use bevy::prelude::*;
use common::components::collider_relations::ChildColliderOf;
use std::collections::HashMap;

pub(crate) fn map_ray_cast_result_to_interaction_events(
	In(results): In<HashMap<Entity, RayCastResult>>,
	roots: Query<&ChildColliderOf>,
) -> Vec<(InteractionEvent<Ray>, Vec<InteractionEvent>)> {
	let mut events = vec![];

	for (entity, RayCastResult { info }) in results.into_iter() {
		let ray = InteractionEvent::of(ChildColliderOf(entity)).ray(info.ray, info.max_toi);
		let mut collisions = vec![];

		let root_entity = get_root(entity, &roots);
		for (hit, ..) in &info.hits {
			let root_hit = get_root(*hit, &roots);
			let event = InteractionEvent::of(root_entity).collision(Collision::Started(root_hit));
			collisions.push(event);
		}

		events.push((ray, collisions));
	}

	events
}

fn get_root(entity: Entity, roots: &Query<&ChildColliderOf>) -> ChildColliderOf {
	match roots.get(entity) {
		Ok(root) => *root,
		Err(_) => ChildColliderOf(entity),
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

		let interaction = InteractionEvent::of(ChildColliderOf(Entity::from_raw(5)));
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
					interaction
						.collision(Collision::Started(ChildColliderOf(Entity::from_raw(42)))),
					interaction
						.collision(Collision::Started(ChildColliderOf(Entity::from_raw(11)))),
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

		let root_a = app.world_mut().spawn_empty().id();
		let root_b = app.world_mut().spawn_empty().id();
		let collider_a = app.world_mut().spawn(ChildColliderOf(root_a)).id();
		let collider_b = app.world_mut().spawn(ChildColliderOf(root_b)).id();
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

		let interaction = InteractionEvent::of(ChildColliderOf(Entity::from_raw(5)));
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
					interaction.collision(Collision::Started(ChildColliderOf(root_a))),
					interaction.collision(Collision::Started(ChildColliderOf(root_b))),
				]
			)],
			events
		);
		Ok(())
	}
}
