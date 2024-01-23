use super::{Model, Offset, Shape};
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

impl<T> Offset for Projectile<T> {
	fn offset() -> Vec3 {
		Vec3::ZERO
	}
}

pub struct Sphere(pub f32);

impl Shape<Sphere> for Projectile<Plasma> {
	fn shape() -> Sphere {
		Sphere(0.05)
	}
}

impl Model<StandardMaterial> for Projectile<Plasma> {
	fn material() -> StandardMaterial {
		StandardMaterial {
			emissive: Color::rgb_linear(2.0, 13.99, 13.99),
			..default()
		}
	}

	fn mesh() -> Mesh {
		Icosphere {
			radius: Self::shape().0,
			subdivisions: 5,
		}
		.try_into()
		.unwrap()
	}
}
