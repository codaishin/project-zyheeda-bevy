use crate::{components::cast_rays::RayCasterArgs, traits::ray_cast::CastRayContinuously};
use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::geometry::Collider};

impl CastRayContinuously<RayCasterArgs> for RapierContext<'_> {
	fn cast_ray_continuously<F: FnMut(Entity, &Collider, RayIntersection) -> bool>(
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
