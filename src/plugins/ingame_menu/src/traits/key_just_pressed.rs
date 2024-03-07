pub mod inventory;

use bevy::{input::ButtonInput, prelude::KeyCode};

pub trait KeyJustPressed {
	fn just_pressed(input: &ButtonInput<KeyCode>) -> bool;
}
