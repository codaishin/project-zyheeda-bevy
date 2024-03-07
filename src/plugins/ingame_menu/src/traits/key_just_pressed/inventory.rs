use super::KeyJustPressed;
use bevy::{input::keyboard::KeyCode, prelude::ButtonInput};
use skills::components::Inventory;

impl KeyJustPressed for Inventory {
	fn just_pressed(input: &ButtonInput<KeyCode>) -> bool {
		input.just_pressed(KeyCode::KeyI)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn true_when_pressed() {
		let mut keys = ButtonInput::<KeyCode>::default();
		keys.press(KeyCode::KeyI);
		assert!(Inventory::just_pressed(&keys));
	}

	#[test]
	fn false_when_not_pressed() {
		let keys = ButtonInput::<KeyCode>::default();
		assert!(!Inventory::just_pressed(&keys));
	}

	#[test]
	fn false_when_not_just_pressed() {
		let mut keys = ButtonInput::<KeyCode>::default();
		keys.press(KeyCode::KeyI);
		keys.clear_just_pressed(KeyCode::KeyI);
		assert!(!Inventory::just_pressed(&keys));
	}
}
