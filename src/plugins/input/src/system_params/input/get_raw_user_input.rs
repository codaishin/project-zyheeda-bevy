use crate::system_params::input::Input;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	tools::action_key::user_input::UserInput,
	traits::handles_input::{GetRawUserInput, RawInputState},
};

impl<'w, 's, TKeyMap> GetRawUserInput for Input<'w, 's, TKeyMap>
where
	TKeyMap: SystemParam + 'static,
{
	fn get_raw_user_input(&self, state: RawInputState) -> impl Iterator<Item = UserInput> {
		let just_pressed = || Iter {
			keys: Some(self.keys.get_just_pressed().copied()),
			mouse: Some(self.mouse.get_just_pressed().copied()),
		};
		let held = || Iter {
			keys: Some(self.keys.get_pressed().copied()),
			mouse: Some(self.mouse.get_pressed().copied()),
		};
		let just_released = || Iter {
			keys: Some(self.keys.get_just_released().copied()),
			mouse: Some(self.mouse.get_just_released().copied()),
		};

		match state {
			RawInputState::JustPressed => just_pressed().chain(Iter::EMPTY).chain(Iter::EMPTY),
			RawInputState::Held => Iter::EMPTY.chain(held()).chain(Iter::EMPTY),
			RawInputState::JustReleased => Iter::EMPTY.chain(Iter::EMPTY).chain(just_released()),
		}
	}
}

struct Iter<TKeys, TMouse> {
	keys: Option<TKeys>,
	mouse: Option<TMouse>,
}

impl<TKeys, TMouse> Iter<TKeys, TMouse>
where
	TKeys: Iterator<Item = KeyCode>,
	TMouse: Iterator<Item = MouseButton>,
{
	const EMPTY: Self = Self {
		keys: None,
		mouse: None,
	};

	fn next_key(&mut self) -> Option<UserInput> {
		self.keys
			.as_mut()
			.and_then(Iterator::next)
			.map(UserInput::KeyCode)
	}

	fn next_mouse(&mut self) -> Option<UserInput> {
		self.mouse
			.as_mut()
			.and_then(Iterator::next)
			.map(UserInput::MouseButton)
	}
}

impl<TKeys: Iterator<Item = KeyCode>, TMouse: Iterator<Item = MouseButton>> Iterator
	for Iter<TKeys, TMouse>
{
	type Item = UserInput;

	fn next(&mut self) -> Option<Self::Item> {
		self.next_key().or_else(|| self.next_mouse())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::{SingleThreadedApp, assert_eq_unordered, set_input};

	#[derive(Resource)]
	struct _Map;

	type _Input<'w, 's> = Input<'w, 's, Res<'static, _Map>>;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(_Map);
		app.init_resource::<ButtonInput<KeyCode>>();
		app.init_resource::<ButtonInput<MouseButton>>();

		app
	}

	#[test]
	fn get_pressed() -> Result<(), RunSystemError> {
		let mut app = setup();
		set_input!(app, pressed(KeyCode::KeyA));
		set_input!(app, just_pressed(KeyCode::KeyB));
		set_input!(app, pressed(MouseButton::Left));
		set_input!(app, just_pressed(MouseButton::Right));

		let input = app.world_mut().run_system_once(|input: _Input| {
			input
				.get_raw_user_input(RawInputState::Held)
				.collect::<Vec<_>>()
		})?;

		assert_eq_unordered!(
			vec![
				UserInput::from(KeyCode::KeyA),
				UserInput::from(KeyCode::KeyB),
				UserInput::from(MouseButton::Left),
				UserInput::from(MouseButton::Right),
			],
			input
		);
		Ok(())
	}

	#[test]
	fn get_only_just_pressed() -> Result<(), RunSystemError> {
		let mut app = setup();
		set_input!(app, pressed(KeyCode::KeyA));
		set_input!(app, just_pressed(KeyCode::KeyB));
		set_input!(app, pressed(MouseButton::Left));
		set_input!(app, just_pressed(MouseButton::Right));

		let input = app.world_mut().run_system_once(|input: _Input| {
			input
				.get_raw_user_input(RawInputState::JustPressed)
				.collect::<Vec<_>>()
		})?;

		assert_eq_unordered!(
			vec![
				UserInput::from(KeyCode::KeyB),
				UserInput::from(MouseButton::Right),
			],
			input
		);
		Ok(())
	}

	#[test]
	fn get_just_released() -> Result<(), RunSystemError> {
		let mut app = setup();
		set_input!(app, released(KeyCode::KeyA));
		set_input!(app, just_released(KeyCode::KeyB));
		set_input!(app, released(MouseButton::Left));
		set_input!(app, just_released(MouseButton::Right));

		let input = app.world_mut().run_system_once(|input: _Input| {
			input
				.get_raw_user_input(RawInputState::JustReleased)
				.collect::<Vec<_>>()
		})?;

		assert_eq_unordered!(
			vec![
				UserInput::from(KeyCode::KeyB),
				UserInput::from(MouseButton::Right),
			],
			input
		);
		Ok(())
	}
}
