use bevy::render::color::Color;

use crate::{
	components::{Bar, Health},
	traits::ui::{UIBarColors, UIBarUpdate},
};

impl UIBarUpdate<Health> for Bar<Health> {
	fn update(&mut self, value: &Health) {
		self.current = value.current as f32;
		self.max = value.max as f32;
	}
}

impl UIBarColors for Bar<Health> {
	fn background_color() -> Color {
		Color::GRAY
	}

	fn foreground_color() -> Color {
		Color::ORANGE_RED
	}
}
