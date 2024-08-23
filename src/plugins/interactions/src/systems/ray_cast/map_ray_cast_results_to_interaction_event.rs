use super::execute_ray_caster::RayCastResult;
use crate::events::{Collision, InteractionEvent, Ray};
use bevy::prelude::{Entity, In, Query};
use common::components::ColliderRoot;
use std::collections::HashMap;

pub(crate) fn map_ray_cast_result_to_interaction_events(
	In(results): In<HashMap<Entity, RayCastResult>>,
	roots: Query<&ColliderRoot>,
) -> Vec<(InteractionEvent<Ray>, Vec<InteractionEvent>)> {
	let mut events = vec![];

	for (entity, RayCastResult { info }) in results.into_iter() {
		let ray = InteractionEvent::of(ColliderRoot(entity)).ray(info.ray, info.max_toi);
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

fn get_root(entity: Entity, roots: &Query<&ColliderRoot>) -> ColliderRoot {
	match roots.get(entity) {
		Ok(root) => *root,
		Err(_) => ColliderRoot(entity),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::RayCastInfo;
	use bevy::{
		app::App,
		ecs::system::RunSystemOnce,
		math::{Ray3d, Vec3},
		prelude::Entity,
	};
	use common::traits::cast_ray::TimeOfImpact;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn build_events() {
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
					ray: Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(5., 6., 7.)),
				},
			},
		)]);

		let events = app
			.world_mut()
			.run_system_once_with(ray_casts, map_ray_cast_result_to_interaction_events);

		let interaction = InteractionEvent::of(ColliderRoot(Entity::from_raw(5)));
		assert_eq!(
			vec![(
				interaction.ray(
					Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(5., 6., 7.)),
					TimeOfImpact(100.)
				),
				vec![
					interaction.collision(Collision::Started(ColliderRoot(Entity::from_raw(42)))),
					interaction.collision(Collision::Started(ColliderRoot(Entity::from_raw(11)))),
				]
			)],
			events
		);
	}

	#[test]
	fn send_event_for_each_target_collision_using_collider_root_reference() {
		let mut app = setup();

		let collider_a = app
			.world_mut()
			.spawn(ColliderRoot(Entity::from_raw(42)))
			.id();
		let collider_b = app
			.world_mut()
			.spawn(ColliderRoot(Entity::from_raw(11)))
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
					ray: Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(5., 6., 7.)),
				},
			},
		)]);

		let events = app
			.world_mut()
			.run_system_once_with(ray_casts, map_ray_cast_result_to_interaction_events);

		let interaction = InteractionEvent::of(ColliderRoot(Entity::from_raw(5)));
		assert_eq!(
			vec![(
				interaction.ray(
					Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(5., 6., 7.)),
					TimeOfImpact(100.)
				),
				vec![
					interaction.collision(Collision::Started(ColliderRoot(Entity::from_raw(42)))),
					interaction.collision(Collision::Started(ColliderRoot(Entity::from_raw(11)))),
				]
			)],
			events
		);
	}
}
