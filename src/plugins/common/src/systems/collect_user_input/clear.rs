use crate::tools::action_key::user_input::UserInput;
use bevy::prelude::*;

impl UserInput {
	pub(crate) fn clear(mut dst: ResMut<ButtonInput<UserInput>>) {
		dst.clear();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;

	fn setup(keys: ButtonInput<UserInput>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(keys);
		app.init_resource::<ButtonInput<UserInput>>();
		app.add_systems(Update, UserInput::clear);

		app
	}

	#[test]
	fn just_pressed_and_released() {
		let mut key_input = ButtonInput::default();
		key_input.press(UserInput::KeyCode(KeyCode::F12));
		key_input.release(UserInput::KeyCode(KeyCode::F12));
		let mut app = setup(key_input);

		app.update();

		let input = app.world().resource::<ButtonInput<UserInput>>();
		assert_eq!(
			(vec![], vec![], vec![]),
			(
				input.get_pressed().collect::<Vec<_>>(),
				input.get_just_pressed().collect::<Vec<_>>(),
				input.get_just_released().collect::<Vec<_>>(),
			),
		);
	}

	#[test]
	fn retain_pressed_keys() {
		let mut key_input = ButtonInput::default();
		key_input.press(UserInput::KeyCode(KeyCode::F12));
		key_input.press(UserInput::KeyCode(KeyCode::F14));
		key_input.release(UserInput::KeyCode(KeyCode::F12));
		let mut app = setup(key_input);

		app.update();

		let input = app.world().resource::<ButtonInput<UserInput>>();
		assert_eq!(
			(vec![&UserInput::KeyCode(KeyCode::F14)], vec![], vec![]),
			(
				input.get_pressed().collect::<Vec<_>>(),
				input.get_just_pressed().collect::<Vec<_>>(),
				input.get_just_released().collect::<Vec<_>>(),
			),
		);
	}
}
