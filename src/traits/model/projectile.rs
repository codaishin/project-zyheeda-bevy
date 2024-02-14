use super::{GetCollider, GetRigidBody, Model, Offset, Shape};
use bevy::{
	math::Vec3,
	pbr::StandardMaterial,
	render::{
		color::Color,
		mesh::{shape::Icosphere, Mesh},
	},
	utils::default,
};
use bevy_rapier3d::{dynamics::RigidBody, geometry::Collider};
use common::components::{Plasma, Projectile};

impl<T> Offset for Projectile<T> {
	fn offset() -> Vec3 {
		Vec3::ZERO
	}
}

impl<T> GetCollider for Projectile<T>
where
	Projectile<T>: Shape<Sphere>,
{
	fn collider() -> bevy_rapier3d::prelude::Collider {
		Collider::ball(Projectile::<T>::shape().0)
	}
}

impl<T> GetRigidBody for Projectile<T> {
	fn rigid_body() -> RigidBody {
		RigidBody::Fixed
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
