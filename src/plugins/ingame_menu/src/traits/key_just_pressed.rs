pub mod inventory;

use bevy::prelude::{Input, KeyCode};

pub trait KeyJustPressed {
	fn just_pressed(input: &Input<KeyCode>) -> bool;
}
