use super::ProjectileBehavior;
use bevy::{self, math::Vec3};
use common::components::Projectile;

impl<T> ProjectileBehavior for Projectile<T> {
	fn direction(&self) -> Vec3 {
		self.direction
	}

	fn range(&self) -> f32 {
		self.range
	}
}
