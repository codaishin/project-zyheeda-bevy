use bevy::{ecs::entity::Entity, math::Ray3d};
use bevy_rapier3d::{math::Real, pipeline::QueryFilterFlags, plugin::RapierContext};

use super::{CastRay, TimeOfImpact};

impl CastRay<Ray3d> for RapierContext {
	fn cast_ray(&self, ray: &Ray3d) -> Option<(Entity, TimeOfImpact)> {
		self.cast_ray(
			ray.origin,
			ray.direction.into(),
			Real::MAX,
			true,
			QueryFilterFlags::EXCLUDE_SENSORS.into(),
		)
		.map(|(entity, toi)| (entity, TimeOfImpact(toi)))
	}
}
