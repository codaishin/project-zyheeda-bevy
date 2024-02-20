use crate::components::RayCaster;
use bevy::ecs::entity::Entity;
use bevy_rapier3d::plugin::RapierContext;
use common::traits::cast_ray::{CastRay, TimeOfImpact};

impl CastRay<RayCaster> for RapierContext {
	fn cast_ray(&self, ray: RayCaster) -> Option<(Entity, TimeOfImpact)> {
		self.cast_ray(
			ray.origin,
			ray.direction,
			ray.max_toi.0,
			ray.solid,
			(ray.get_filter)(),
		)
		.map(|(entity, toi)| (entity, TimeOfImpact(toi)))
	}
}
