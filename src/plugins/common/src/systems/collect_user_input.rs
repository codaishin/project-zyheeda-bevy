use crate::{tools::action_key::user_input::UserInput, traits::thread_safe::ThreadSafe};
use bevy::prelude::*;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct InputSystem;

impl UserInput {
	pub const SYSTEM: InputSystem = InputSystem;

	pub(crate) fn clear(mut dst: ResMut<ButtonInput<UserInput>>) {
		dst.clear();
	}

	pub(crate) fn collect<T>(src: Res<ButtonInput<T>>, mut dst: ResMut<ButtonInput<UserInput>>)
	where
		T: Into<UserInput> + Eq + Hash + Copy + ThreadSafe,
	{
		for key in src.get_just_pressed() {
			let user_input = (*key).into();
			dst.press(user_input);
		}

		for key in src.get_just_released() {
			let user_input = (*key).into();
			dst.release(user_input);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;

	mod clear {
		use super::*;

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

	mod collect {
		use super::*;

		#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
		struct _T(UserInput);

		impl From<_T> for UserInput {
			fn from(_T(user_input): _T) -> Self {
				user_input
			}
		}

		fn setup(keys: ButtonInput<_T>) -> App {
			let mut app = App::new().single_threaded(Update);

			app.insert_resource(keys);
			app.init_resource::<ButtonInput<UserInput>>();
			app.add_systems(Update, UserInput::collect::<_T>);

			app
		}

		#[test]
		fn just_pressed() {
			let mut key_input = ButtonInput::default();
			key_input.press(_T(UserInput::KeyCode(KeyCode::F12)));
			let mut app = setup(key_input);

			app.update();

			let input = app.world().resource::<ButtonInput<UserInput>>();
			assert_eq!(
				(
					vec![&UserInput::KeyCode(KeyCode::F12)],
					vec![&UserInput::KeyCode(KeyCode::F12)],
					vec![]
				),
				(
					input.get_pressed().collect::<Vec<_>>(),
					input.get_just_pressed().collect::<Vec<_>>(),
					input.get_just_released().collect::<Vec<_>>(),
				),
			);
		}

		#[test]
		fn just_released() {
			let mut key_input = ButtonInput::default();
			key_input.press(_T(UserInput::KeyCode(KeyCode::F12)));
			key_input.release(_T(UserInput::KeyCode(KeyCode::F12)));
			let mut app = setup(key_input);

			app.update();

			let input = app.world().resource::<ButtonInput<UserInput>>();
			assert_eq!(
				(
					vec![],
					vec![&UserInput::KeyCode(KeyCode::F12)],
					vec![&UserInput::KeyCode(KeyCode::F12)]
				),
				(
					input.get_pressed().collect::<Vec<_>>(),
					input.get_just_pressed().collect::<Vec<_>>(),
					input.get_just_released().collect::<Vec<_>>(),
				),
			);
		}
	}
}
