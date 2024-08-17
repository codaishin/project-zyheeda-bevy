use crate::components::RayCaster;
use bevy::ecs::entity::Entity;
use bevy_rapier3d::{pipeline::QueryFilter, plugin::RapierContext, prelude::RayIntersection};
use common::traits::cast_ray::{CastRay, CastRayContinuously, TimeOfImpact};

impl CastRay<RayCaster> for RapierContext {
	fn cast_ray(&self, ray: &RayCaster) -> Option<(Entity, TimeOfImpact)> {
		self.cast_ray(
			ray.origin,
			ray.direction.into(),
			ray.max_toi.0,
			ray.solid,
			QueryFilter::from(ray.filter.clone()),
		)
		.map(|(entity, toi)| (entity, TimeOfImpact(toi)))
	}
}

impl CastRayContinuously<RayCaster> for RapierContext {
	fn cast_ray_continuously(&self, ray: &RayCaster) -> Vec<(Entity, TimeOfImpact)> {
		let mut results = Vec::new();

		self.intersections_with_ray(
			ray.origin,
			ray.direction.into(),
			ray.max_toi.0,
			ray.solid,
			QueryFilter::from(ray.filter.clone()),
			|entity, RayIntersection { time_of_impact, .. }| {
				results.push((entity, TimeOfImpact(time_of_impact)));
				true
			},
		);

		results.sort_by(|(_, toi_a), (_, toi_b)| {
			// FIXME: needs a more stable solution
			toi_a.partial_cmp(toi_b).unwrap_or_else(|| {
				panic!("tried to sort non comparable {:?} vs {:?}", toi_a, toi_b)
			})
		});

		results
	}
}
