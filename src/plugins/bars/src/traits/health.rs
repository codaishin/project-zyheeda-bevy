use crate::{
	components::BarValues,
	traits::{UIBarColors, UIBarUpdate},
};
use bevy::color::Color;
use common::components::Health;

impl UIBarUpdate<Health> for BarValues<Health> {
	fn update(&mut self, value: &Health) {
		self.current = value.current as f32;
		self.max = value.max as f32;
	}
}

impl UIBarColors for BarValues<Health> {
	fn background_color() -> Color {
		Color::srgb(0.5, 0.5, 0.5)
	}

	fn foreground_color() -> Color {
		Color::srgb(1., 0.27, 0.)
	}
}
