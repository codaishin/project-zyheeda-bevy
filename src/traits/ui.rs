pub mod health;

use bevy::math::Vec3;

pub trait UIBarOffset<T> {
	fn ui_bar_offset() -> Vec3;
}

pub trait UIBarScale<T> {
	fn ui_bar_scale() -> f32;
}

pub trait UIBarUpdate<T> {
	fn update(&mut self, value: &T);
}
