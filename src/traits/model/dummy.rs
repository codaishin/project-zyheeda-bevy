use super::{Model, Offset, Shape};
use crate::components::Dummy;
use bevy::{
	math::Vec3,
	pbr::StandardMaterial,
	render::{
		color::Color,
		mesh::{
			shape::{self},
			Mesh,
		},
	},
	utils::default,
};

pub struct Cuboid(pub Vec3);

impl Shape<Cuboid> for Dummy {
	fn shape() -> Cuboid {
		Cuboid(Vec3::new(0.4, 2., 0.4))
	}
}

impl Offset for Dummy {
	fn offset() -> Vec3 {
		Vec3::new(0., 1., 0.)
	}
}

impl Model<StandardMaterial> for Dummy {
	fn material() -> StandardMaterial {
		StandardMaterial {
			base_color: Color::GRAY,
			..default()
		}
	}

	fn mesh() -> Mesh {
		let dimensions = Self::shape().0;
		shape::Box::new(dimensions.x, dimensions.y, dimensions.z).into()
	}
}
