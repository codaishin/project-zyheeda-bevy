use crate::system_params::input::Input;
use bevy::{ecs::system::SystemParam, input::ButtonInput, prelude::*};
use common::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{
		handles_input::{GetInput, GetInputState, InputState},
		thread_safe::ThreadSafe,
	},
};
use std::hash::Hash;

impl<'w, 's, TKeyMap> GetInputState for Input<'w, 's, TKeyMap>
where
	TKeyMap: SystemParam<Item<'w, 's>: GetInput> + 'static,
{
	fn get_input_state<TAction>(&self, action: TAction) -> InputState
	where
		TAction: Into<ActionKey> + 'static,
	{
		match self.key_map.get_input(action) {
			UserInput::KeyCode(key_code) => get_input_state(&self.keys, key_code),
			UserInput::MouseButton(mouse_button) => get_input_state(&self.mouse, mouse_button),
		}
	}
}

fn get_input_state<T>(input: &Res<ButtonInput<T>>, button: T) -> InputState
where
	T: Copy + Eq + Hash + ThreadSafe,
{
	if input.just_pressed(button) {
		return InputState::just_pressed();
	}

	if input.pressed(button) {
		return InputState::pressed();
	}

	if input.just_released(button) {
		return InputState::just_released();
	}

	InputState::released()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::action_key::user_input::UserInput;
	use macros::NestedMocks;
	use mockall::automock;
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp, set_input};

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl GetInput for _Map {
		fn get_input<TAction>(&self, action: TAction) -> UserInput
		where
			TAction: Into<ActionKey> + 'static,
		{
			self.mock.get_input(action)
		}
	}

	struct _Action;

	impl From<_Action> for ActionKey {
		fn from(_: _Action) -> Self {
			panic!("DO NOT USE")
		}
	}

	type _Input<'w, 's> = Input<'w, 's, Res<'static, _Map>>;

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.init_resource::<ButtonInput<KeyCode>>();
		app.init_resource::<ButtonInput<MouseButton>>();

		app
	}

	#[test_case(KeyCode::KeyA; "key")]
	#[test_case(MouseButton::Right; "mouse button")]
	fn get_pressed<TInput>(user_input: TInput) -> Result<(), RunSystemError>
	where
		TInput: Into<UserInput> + Copy + Eq + Hash + ThreadSafe,
	{
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_get_input::<_Action>()
				.return_const(user_input.into());
		}));
		set_input!(app, pressed(user_input));

		let state = app
			.world_mut()
			.run_system_once(|input: _Input| input.get_input_state(_Action))?;

		assert_eq!(InputState::pressed(), state);
		Ok(())
	}

	#[test_case(KeyCode::KeyA; "key")]
	#[test_case(MouseButton::Right; "mouse button")]
	fn get_just_pressed<TInput>(user_input: TInput) -> Result<(), RunSystemError>
	where
		TInput: Into<UserInput> + Copy + Eq + Hash + ThreadSafe,
	{
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_get_input::<_Action>()
				.return_const(user_input.into());
		}));
		set_input!(app, just_pressed(user_input));

		let state = app
			.world_mut()
			.run_system_once(|input: _Input| input.get_input_state(_Action))?;

		assert_eq!(InputState::just_pressed(), state);
		Ok(())
	}

	#[test_case(KeyCode::KeyA; "key")]
	#[test_case(MouseButton::Right; "mouse button")]
	fn get_released<TInput>(user_input: TInput) -> Result<(), RunSystemError>
	where
		TInput: Into<UserInput> + Copy + Eq + Hash + ThreadSafe,
	{
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_get_input::<_Action>()
				.return_const(user_input.into());
		}));
		set_input!(app, released(user_input));

		let state = app
			.world_mut()
			.run_system_once(|input: _Input| input.get_input_state(_Action))?;

		assert_eq!(InputState::released(), state);
		Ok(())
	}

	#[test_case(KeyCode::KeyA; "key")]
	#[test_case(MouseButton::Right; "mouse button")]
	fn get_just_released<TInput>(user_input: TInput) -> Result<(), RunSystemError>
	where
		TInput: Into<UserInput> + Copy + Eq + Hash + ThreadSafe,
	{
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_get_input::<_Action>()
				.return_const(user_input.into());
		}));
		set_input!(app, just_released(user_input));

		let state = app
			.world_mut()
			.run_system_once(|input: _Input| input.get_input_state(_Action))?;

		assert_eq!(InputState::just_released(), state);
		Ok(())
	}
}
