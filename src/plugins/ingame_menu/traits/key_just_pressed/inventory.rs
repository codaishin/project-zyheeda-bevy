use super::KeyJustPressed;
use crate::components::Inventory;
use bevy::{input::keyboard::KeyCode, prelude::Input};

impl KeyJustPressed for Inventory {
	fn just_pressed(input: &Input<KeyCode>) -> bool {
		input.just_pressed(KeyCode::I)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{input::keyboard::KeyCode, prelude::Input};

	#[test]
	fn true_when_pressed() {
		let mut keys = Input::<KeyCode>::default();
		keys.press(KeyCode::I);
		assert!(Inventory::just_pressed(&keys));
	}

	#[test]
	fn false_when_not_pressed() {
		let keys = Input::<KeyCode>::default();
		assert!(!Inventory::just_pressed(&keys));
	}

	#[test]
	fn false_when_not_just_pressed() {
		let mut keys = Input::<KeyCode>::default();
		keys.press(KeyCode::I);
		keys.clear_just_pressed(KeyCode::I);
		assert!(!Inventory::just_pressed(&keys));
	}
}
