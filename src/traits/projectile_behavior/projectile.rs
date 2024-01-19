use super::ProjectileBehavior;
use crate::components::Projectile;
use bevy::{self, math::Vec3};

impl<T> ProjectileBehavior for Projectile<T> {
	fn direction(&self) -> Vec3 {
		self.direction
	}

	fn range(&self) -> f32 {
		self.range
	}
}
