use super::{ProjectileBehaviorData, ProjectileModelData};
use crate::components::{Plasma, Projectile};
use bevy::{
	math::Vec3,
	pbr::StandardMaterial,
	render::{
		color::Color,
		mesh::{shape::Icosphere, Mesh},
	},
	utils::default,
};

impl ProjectileModelData for Projectile<Plasma> {
	fn material() -> StandardMaterial {
		StandardMaterial {
			emissive: Color::rgb_linear(2.0, 13.99, 13.99),
			..default()
		}
	}

	fn mesh() -> Mesh {
		Icosphere {
			radius: 0.05,
			subdivisions: 5,
		}
		.try_into()
		.unwrap()
	}
}

impl<T> ProjectileBehaviorData for Projectile<T> {
	fn direction(&self) -> Vec3 {
		self.direction
	}

	fn range(&self) -> f32 {
		self.range
	}
}
