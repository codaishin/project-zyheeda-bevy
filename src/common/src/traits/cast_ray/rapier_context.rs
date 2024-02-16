use bevy::{ecs::entity::Entity, math::Ray};
use bevy_rapier3d::{math::Real, pipeline::QueryFilter, plugin::RapierContext};

use super::{CastRay, TimeOfImpact};

impl CastRay for RapierContext {
	fn cast_ray(&self, ray: Ray) -> Option<(Entity, TimeOfImpact)> {
		self.cast_ray(
			ray.origin,
			ray.direction,
			Real::MAX,
			true,
			QueryFilter::default(),
		)
		.map(|(entity, toi)| (entity, TimeOfImpact(toi)))
	}
}
