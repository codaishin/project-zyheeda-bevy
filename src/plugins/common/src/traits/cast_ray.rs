pub mod rapier_context;

use bevy::ecs::entity::Entity;
use bevy_rapier3d::math::Real;

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct TimeOfImpact(pub Real);

pub trait CastRay<TRayData> {
	fn cast_ray(&self, ray: TRayData) -> Option<(Entity, TimeOfImpact)>;
}
