use crate::{components::RayCasterArgs, traits::cast_ray::CastRayContinuously};
use bevy::ecs::entity::Entity;
use bevy_rapier3d::{pipeline::QueryFilter, plugin::RapierContext, prelude::RayIntersection};

impl CastRayContinuously<RayCasterArgs> for RapierContext<'_> {
	fn cast_ray_continuously<F: FnMut(Entity, RayIntersection) -> bool>(
		&self,
		ray: &RayCasterArgs,
		callback: F,
	) {
		self.intersect_ray(
			ray.origin,
			ray.direction.into(),
			*ray.max_toi,
			ray.solid,
			QueryFilter::from(ray.filter.clone()),
			callback,
		);
	}
}
