use super::Model;
use crate::components::{Plasma, Projectile};
use bevy::{
	pbr::StandardMaterial,
	render::{
		color::Color,
		mesh::{shape::Icosphere, Mesh},
	},
	utils::default,
};

impl Model<StandardMaterial> for Projectile<Plasma> {
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
