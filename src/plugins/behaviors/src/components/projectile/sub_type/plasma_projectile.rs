use super::traits::ProjectileTypeParameters;
use bevy::color::Color;
use common::{
	tools::{Intensity, Units},
	traits::clamp_zero_positive::ClampZeroPositive,
};

pub(super) struct PlasmaProjectile;

impl ProjectileTypeParameters for PlasmaProjectile {
	fn radius() -> Units {
		Units::new(0.05)
	}

	fn base_color() -> Color {
		Color::WHITE
	}

	fn emissive() -> (Color, Intensity) {
		(Color::srgb(0., 1., 1.), Intensity::new(2300.0))
	}
}
