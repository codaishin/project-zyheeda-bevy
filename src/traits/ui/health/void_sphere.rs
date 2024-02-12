use crate::{
	components::Health,
	traits::ui::{UIBarOffset, UIBarScale},
};
use bevy::math::Vec3;
use common::components::VoidSphere;

impl UIBarOffset<Health> for VoidSphere {
	fn ui_bar_offset() -> Vec3 {
		Vec3::new(0., 2., 0.)
	}
}

impl UIBarScale<Health> for VoidSphere {
	fn ui_bar_scale() -> f32 {
		1.
	}
}
