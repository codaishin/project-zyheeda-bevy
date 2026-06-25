pub mod health;

use bevy::prelude::*;

pub trait UIBarUpdate<T> {
	fn update(&mut self, value: &T);
}

pub trait UIBarColors {
	fn background_color() -> Color;
	fn foreground_color() -> Color;
}
