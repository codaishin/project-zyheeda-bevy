pub mod projectile;

use bevy::math::Vec3;

pub trait ProjectileBehavior {
	fn direction(&self) -> Vec3;
	fn range(&self) -> f32;
}
