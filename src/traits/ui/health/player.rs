use crate::{
	components::{Health, Player},
	traits::ui::{UIBarOffset, UIBarScale},
};
use bevy::math::Vec3;

impl UIBarOffset<Health> for Player {
	fn ui_bar_offset() -> Vec3 {
		Vec3::new(0., 2., 0.)
	}
}

impl UIBarScale<Health> for Player {
	fn ui_bar_scale() -> f32 {
		1.
	}
}
