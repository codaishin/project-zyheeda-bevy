use crate::{
	components::bar_values::BarValues,
	traits::{UIBarColors, UIBarUpdate},
};
use bevy::color::Color;
use common::attributes::health::Health;

impl UIBarUpdate<Health> for BarValues<Health> {
	fn update(&mut self, value: &Health) {
		self.current = value.current;
		self.max = value.max;
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
