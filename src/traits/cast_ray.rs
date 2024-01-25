pub mod rapier_context;

use bevy::{ecs::entity::Entity, math::Ray};
use bevy_rapier3d::math::Real;

#[derive(Clone)]
pub struct TimeOfImpact(pub Real);

pub trait CastRay {
	fn cast_ray(&self, ray: Ray) -> Option<(Entity, TimeOfImpact)>;
}
