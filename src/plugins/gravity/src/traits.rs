use bevy_rapier3d::geometry::Collider;
use common::tools::UnitsPerSecond;

pub trait GetGravityPull {
	fn gravity_pull(&self) -> UnitsPerSecond;
}

pub trait GetGravityEffectCollider {
	fn gravity_effect_collider(&self) -> Collider;
}
