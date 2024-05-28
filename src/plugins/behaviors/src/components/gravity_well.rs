use bevy::ecs::component::Component;
use bevy_rapier3d::geometry::Collider;
use common::{tools::UnitsPerSecond, traits::clamp_zero_positive::ClampZeroPositive};
use gravity::traits::{GetGravityEffectCollider, GetGravityPull};

#[derive(Component, Debug, PartialEq)]
pub struct GravityWell;

impl GravityWell {
	const RADIUS: f32 = 2.;
}

impl GetGravityPull for GravityWell {
	fn gravity_pull(&self) -> UnitsPerSecond {
		UnitsPerSecond::new(2.)
	}
}
impl GetGravityEffectCollider for GravityWell {
	fn gravity_effect_collider(&self) -> bevy_rapier3d::prelude::Collider {
		Collider::ball(GravityWell::RADIUS)
	}
}
