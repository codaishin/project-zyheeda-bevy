use super::Model;
use crate::components::Dummy;
use bevy::{
	math::Vec3,
	pbr::StandardMaterial,
	render::{color::Color, mesh::shape::Box},
	utils::default,
};

impl Model<StandardMaterial> for Dummy {
	fn material() -> StandardMaterial {
		StandardMaterial {
			base_color: Color::GRAY,
			..default()
		}
	}

	fn mesh() -> bevy::prelude::Mesh {
		Box::from_corners(Vec3::new(-0.2, 0., -0.2), Vec3::new(0.2, 2., 0.2)).into()
	}
}
