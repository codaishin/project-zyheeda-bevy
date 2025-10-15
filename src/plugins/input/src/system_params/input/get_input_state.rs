use crate::{resources::mouse_override::MouseOverride, system_params::input::Input};
use bevy::{ecs::system::SystemParam, input::ButtonInput, prelude::*};
use common::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{
		handles_input::{GetInput, GetInputState, InputState},
		thread_safe::ThreadSafe,
	},
};
use std::{hash::Hash, ops::Deref};

impl<'w, 's, TKeyMap> GetInputState for Input<'w, 's, TKeyMap>
where
	TKeyMap: SystemParam<Item<'w, 's>: GetInput> + 'static,
{
	fn get_input_state<TAction>(&self, action: TAction) -> InputState
	where
		TAction: Into<ActionKey> + 'static,
	{
		let action_key = action.into();

		if let Some(input_state) = self.matching_mouse_override(action_key) {
			return input_state;
		}

		let input = self.key_map.get_input(action_key);

		if input == LEFT_MOUSE && self.mouse_override_active() {
			return InputState::released();
		}

		match input {
			UserInput::KeyCode(key_code) => get_input_state(&self.keys, key_code),
			UserInput::MouseButton(mouse_button) => get_input_state(&self.mouse, mouse_button),
		}
	}
}

impl<'w, 's, TKeyMap> Input<'w, 's, TKeyMap>
where
	TKeyMap: SystemParam,
{
	fn mouse_override_active(&self) -> bool {
		*self.mouse_override != MouseOverride::Idle
	}

	fn matching_mouse_override(&self, action_key: ActionKey) -> Option<InputState> {
		let MouseOverride::World {
			action,
			input_state,
			..
		} = self.mouse_override.deref()
		else {
			return None;
		};

		if &action_key != action {
			return None;
		}

		Some(*input_state)
	}
}

const LEFT_MOUSE: UserInput = UserInput::MouseButton(MouseButton::Left);

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

	#[derive(Default)]
	struct _Action(ActionKey);

	impl From<_Action> for ActionKey {
		fn from(_Action(action_key): _Action) -> Self {
			action_key
		}
	}

	type _Input<'w, 's> = Input<'w, 's, Res<'static, _Map>>;

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.init_resource::<MouseOverride>();
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
			mock.expect_get_input::<ActionKey>()
				.return_const(user_input.into());
		}));
		set_input!(app, pressed(user_input));

		let state = app
			.world_mut()
			.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

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
			mock.expect_get_input::<ActionKey>()
				.return_const(user_input.into());
		}));
		set_input!(app, just_pressed(user_input));

		let state = app
			.world_mut()
			.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

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
			mock.expect_get_input::<ActionKey>()
				.return_const(user_input.into());
		}));
		set_input!(app, released(user_input));

		let state = app
			.world_mut()
			.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

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
			mock.expect_get_input::<ActionKey>()
				.return_const(user_input.into());
		}));
		set_input!(app, just_released(user_input));

		let state = app
			.world_mut()
			.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

		assert_eq!(InputState::just_released(), state);
		Ok(())
	}

	mod mouse_primer {
		use super::*;
		use common::tools::action_key::slot::PlayerSlot;
		use test_case::test_case;

		const LEFT_MOUSE: MouseButton = MouseButton::Left;

		#[test]
		fn ignore_left_mouse_button_when_mouse_overridden_for_ui() -> Result<(), RunSystemError> {
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(LEFT_MOUSE);
			}));
			app.insert_resource(MouseOverride::Ui {
				panel: Entity::from_raw(42),
			});
			set_input!(app, pressed(LEFT_MOUSE));

			let state = app
				.world_mut()
				.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

			assert_eq!(InputState::released(), state);
			Ok(())
		}

		#[test]
		fn ignore_left_mouse_button_when_mouse_overridden_for_world() -> Result<(), RunSystemError>
		{
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(LEFT_MOUSE);
			}));
			app.insert_resource(MouseOverride::World {
				panel: Entity::from_raw(42),
				action: ActionKey::from(PlayerSlot::LOWER_L),
				input_state: InputState::just_pressed(),
			});
			set_input!(app, pressed(LEFT_MOUSE));

			let state = app.world_mut().run_system_once(|input: _Input| {
				input.get_input_state(ActionKey::from(PlayerSlot::UPPER_R))
			})?;

			assert_eq!(InputState::released(), state);
			Ok(())
		}

		#[test]
		fn use_left_mouse_button_when_mouse_override_idle() -> Result<(), RunSystemError> {
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>()
					.return_const(LEFT_MOUSE);
			}));
			app.insert_resource(MouseOverride::Idle);
			set_input!(app, pressed(LEFT_MOUSE));

			let state = app
				.world_mut()
				.run_system_once(|input: _Input| input.get_input_state(_Action::default()))?;

			assert_eq!(InputState::pressed(), state);
			Ok(())
		}

		#[test_case(InputState::just_pressed(); "just pressed")]
		#[test_case(InputState::pressed(); "pressed")]
		#[test_case(InputState::just_released(); "just released")]
		fn return_override_key_input_state(input_state: InputState) -> Result<(), RunSystemError> {
			let action = ActionKey::from(PlayerSlot::UPPER_L);
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_get_input::<ActionKey>().never();
			}));
			app.insert_resource(MouseOverride::World {
				panel: Entity::from_raw(123),
				action,
				input_state,
			});

			let state = app
				.world_mut()
				.run_system_once(move |input: _Input| input.get_input_state(action))?;

			assert_eq!(input_state, state);
			Ok(())
		}
	}
}
