use crate::components::RayCasterArgs;
use bevy::ecs::entity::Entity;
use bevy_rapier3d::{pipeline::QueryFilter, plugin::RapierContext, prelude::RayIntersection};
use common::traits::cast_ray::{CastRay, CastRayContinuously, TimeOfImpact};

impl CastRay<RayCasterArgs> for RapierContext<'_> {
	fn cast_ray(&self, ray: &RayCasterArgs) -> Option<(Entity, TimeOfImpact)> {
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

impl CastRayContinuously<RayCasterArgs> for RapierContext<'_> {
	fn cast_ray_continuously<F: FnMut(Entity, RayIntersection) -> bool>(
		&self,
		ray: &RayCasterArgs,
		callback: F,
	) {
		self.intersections_with_ray(
			ray.origin,
			ray.direction.into(),
			ray.max_toi.0,
			ray.solid,
			QueryFilter::from(ray.filter.clone()),
			callback,
		);
	}
}
