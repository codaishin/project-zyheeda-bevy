use crate::{
	components::ui_input_primer::KeyPrimed,
	tools::action_key::user_input::UserInput,
	traits::thread_safe::ThreadSafe,
};
use bevy::prelude::*;
use std::hash::Hash;

impl UserInput {
	pub(crate) fn collect<TInput, TPrimed>(
		src: Res<ButtonInput<TInput>>,
		mut dst: ResMut<ButtonInput<UserInput>>,
		primers: Query<&TPrimed>,
	) where
		TInput: Into<UserInput> + Eq + Hash + Copy + ThreadSafe,
		TPrimed: KeyPrimed + Component,
	{
		let is_not_primed =
			|user_input: &UserInput| !primers.iter().any(|primer| primer.key_primed(user_input));
		let just_pressed = src
			.get_just_pressed()
			.copied()
			.map(TInput::into)
			.filter(is_not_primed);
		let just_released = src
			.get_just_released()
			.copied()
			.map(TInput::into)
			.filter(is_not_primed);

		for user_input in just_pressed {
			dst.press(user_input);
		}

		for user_input in just_released {
			dst.release(user_input);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;
	use bevy::utils::HashSet;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _T(UserInput);

	impl From<_T> for UserInput {
		fn from(_T(user_input): _T) -> Self {
			user_input
		}
	}

	#[derive(Component)]
	struct _Primer {
		key: UserInput,
		primed: bool,
	}

	impl KeyPrimed for _Primer {
		fn key_primed(&self, key: &UserInput) -> bool {
			self.primed && &self.key == key
		}
	}

	fn setup(keys: ButtonInput<_T>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(keys);
		app.init_resource::<ButtonInput<UserInput>>();
		app.add_systems(Update, UserInput::collect::<_T, _Primer>);

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

	#[test]
	fn ignore_primed_in_just_pressed() {
		let mut key_input = ButtonInput::default();
		key_input.press(_T(UserInput::KeyCode(KeyCode::F12)));
		key_input.press(_T(UserInput::KeyCode(KeyCode::F11)));
		key_input.press(_T(UserInput::KeyCode(KeyCode::F10)));
		let mut app = setup(key_input);
		app.world_mut().spawn(_Primer {
			key: UserInput::KeyCode(KeyCode::F12),
			primed: false,
		});
		app.world_mut().spawn(_Primer {
			key: UserInput::KeyCode(KeyCode::F10),
			primed: true,
		});

		app.update();

		let input = app.world().resource::<ButtonInput<UserInput>>();
		assert_eq!(
			(
				HashSet::from([
					&UserInput::KeyCode(KeyCode::F12),
					&UserInput::KeyCode(KeyCode::F11),
				]),
				HashSet::from([
					&UserInput::KeyCode(KeyCode::F12),
					&UserInput::KeyCode(KeyCode::F11),
				]),
				HashSet::from([]),
			),
			(
				input.get_pressed().collect::<HashSet<_>>(),
				input.get_just_pressed().collect::<HashSet<_>>(),
				input.get_just_released().collect::<HashSet<_>>(),
			),
		);
	}

	#[test]
	fn ignore_primed_in_just_released() {
		let mut key_input = ButtonInput::default();
		key_input.press(_T(UserInput::KeyCode(KeyCode::F12)));
		key_input.press(_T(UserInput::KeyCode(KeyCode::F11)));
		key_input.press(_T(UserInput::KeyCode(KeyCode::F10)));
		let mut app = setup(key_input);

		app.update();
		app.world_mut().spawn(_Primer {
			key: UserInput::KeyCode(KeyCode::F12),
			primed: false,
		});
		app.world_mut().spawn(_Primer {
			key: UserInput::KeyCode(KeyCode::F10),
			primed: true,
		});
		let mut key_input = app.world_mut().resource_mut::<ButtonInput<_T>>();
		key_input.release(_T(UserInput::KeyCode(KeyCode::F12)));
		key_input.release(_T(UserInput::KeyCode(KeyCode::F11)));
		key_input.release(_T(UserInput::KeyCode(KeyCode::F10)));
		app.update();

		let input = app.world().resource::<ButtonInput<UserInput>>();
		assert_eq!(
			(
				HashSet::from([&UserInput::KeyCode(KeyCode::F10)]),
				HashSet::from([
					&UserInput::KeyCode(KeyCode::F12),
					&UserInput::KeyCode(KeyCode::F11),
					&UserInput::KeyCode(KeyCode::F10),
				]),
				HashSet::from([
					&UserInput::KeyCode(KeyCode::F12),
					&UserInput::KeyCode(KeyCode::F11),
				]),
			),
			(
				input.get_pressed().collect::<HashSet<_>>(),
				input.get_just_pressed().collect::<HashSet<_>>(),
				input.get_just_released().collect::<HashSet<_>>(),
			),
		);
	}
}
