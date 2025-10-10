use crate::{resources::key_map::KeyMap, traits::get_user_input_state::GetUserInputState};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{
		handles_input::{GetActionInputState, InputState, UpdateKey},
		key_mappings::GetInput,
	},
};

#[derive(SystemParam)]
pub struct Input<'w, TKeyMap = KeyMap, TButtonInput = ButtonInput<UserInput>>
where
	TKeyMap: Resource,
	TButtonInput: Resource,
{
	pub(crate) key_map: Res<'w, TKeyMap>,
	pub(crate) input: Res<'w, TButtonInput>,
}

impl<TKeyMap, TButtonInput> GetActionInputState for Input<'_, TKeyMap, TButtonInput>
where
	TKeyMap: Resource + GetInput,
	TButtonInput: Resource + GetUserInputState,
{
	fn get_action_input_state<TAction>(&self, action: TAction) -> InputState
	where
		TAction: Into<ActionKey> + 'static,
	{
		let input_key = self.key_map.get_input(action);
		self.input.get_user_input_state(input_key)
	}
}

#[derive(SystemParam)]
pub struct InputMut<'w, TKeyMap = KeyMap, TButtonInput = ButtonInput<UserInput>>
where
	TKeyMap: Resource,
	TButtonInput: Resource,
{
	key_map: ResMut<'w, TKeyMap>,
	pub(crate) input: Res<'w, TButtonInput>,
}

impl<TKeyMap> UpdateKey for InputMut<'_, TKeyMap>
where
	TKeyMap: Resource + UpdateKey,
{
	fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
	where
		TAction: Copy + Into<ActionKey> + 'static,
	{
		self.key_map.update_key(action, input);
	}
}

impl<TKeyMap, TButtonInput> GetActionInputState for InputMut<'_, TKeyMap, TButtonInput>
where
	TKeyMap: Resource + GetInput,
	TButtonInput: Resource + GetUserInputState,
{
	fn get_action_input_state<TAction>(&self, action: TAction) -> InputState
	where
		TAction: Into<ActionKey> + 'static,
	{
		let input_key = self.key_map.get_input(action);
		self.input.get_user_input_state(input_key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::action_key::slot::PlayerSlot;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	mod update_key {
		use super::*;

		#[derive(Resource, NestedMocks)]
		struct _Map {
			mock: Mock_Map,
		}

		fn setup(map: _Map) -> App {
			let mut app = App::new().single_threaded(Update);

			app.init_resource::<ButtonInput<UserInput>>();
			app.insert_resource(map);

			app
		}

		#[automock]
		impl UpdateKey for _Map {
			fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
			where
				TAction: Copy + Into<ActionKey> + 'static,
			{
				self.mock.update_key(action, input);
			}
		}

		#[test]
		fn call_update_key() -> Result<(), RunSystemError> {
			let mut app = setup(_Map::new().with_mock(|mock| {
				mock.expect_update_key()
					.times(1)
					.with(
						eq(PlayerSlot::LOWER_L),
						eq(UserInput::MouseButton(MouseButton::Left)),
					)
					.return_const(());
			}));

			app.world_mut()
				.run_system_once(|mut input: InputMut<_Map>| {
					input.update_key(
						PlayerSlot::LOWER_L,
						UserInput::MouseButton(MouseButton::Left),
					);
				})
		}
	}

	mod get_input_state {
		use super::*;

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

		#[derive(Resource, NestedMocks)]
		struct _UserInput {
			mock: Mock_UserInput,
		}

		#[automock]
		impl GetUserInputState for _UserInput {
			fn get_user_input_state(&self, user_input: UserInput) -> InputState {
				self.mock.get_user_input_state(user_input)
			}
		}

		fn setup(map: _Map, user_input: _UserInput) -> App {
			let mut app = App::new().single_threaded(Update);

			app.insert_resource(user_input);
			app.insert_resource(map);

			app
		}

		mod input {
			use super::*;

			#[test]
			fn return_state() -> Result<(), RunSystemError> {
				let mut app = setup(
					_Map::new().with_mock(|mock| {
						mock.expect_get_input::<PlayerSlot>()
							.return_const(UserInput::KeyCode(KeyCode::Enter));
					}),
					_UserInput::new().with_mock(|mock| {
						mock.expect_get_user_input_state()
							.return_const(InputState::Pressed { just_now: false });
					}),
				);

				let state = app
					.world_mut()
					.run_system_once(|input: Input<_Map, _UserInput>| {
						input.get_action_input_state(PlayerSlot::UPPER_R)
					})?;

				assert_eq!(InputState::Pressed { just_now: false }, state);
				Ok(())
			}

			#[test]
			fn use_correct_user_input() -> Result<(), RunSystemError> {
				let mut app = setup(
					_Map::new().with_mock(|mock| {
						mock.expect_get_input::<PlayerSlot>()
							.return_const(UserInput::KeyCode(KeyCode::Enter));
					}),
					_UserInput::new().with_mock(|mock| {
						mock.expect_get_user_input_state()
							.times(1)
							.with(eq(UserInput::KeyCode(KeyCode::Enter)))
							.return_const(InputState::Pressed { just_now: false });
					}),
				);

				app.world_mut()
					.run_system_once(|input: Input<_Map, _UserInput>| {
						input.get_action_input_state(PlayerSlot::UPPER_R);
					})
			}
		}

		mod input_mut {
			use super::*;

			#[test]
			fn return_state() -> Result<(), RunSystemError> {
				let mut app = setup(
					_Map::new().with_mock(|mock| {
						mock.expect_get_input::<PlayerSlot>()
							.return_const(UserInput::KeyCode(KeyCode::Enter));
					}),
					_UserInput::new().with_mock(|mock| {
						mock.expect_get_user_input_state()
							.return_const(InputState::Pressed { just_now: false });
					}),
				);

				let state =
					app.world_mut()
						.run_system_once(|input: InputMut<_Map, _UserInput>| {
							input.get_action_input_state(PlayerSlot::UPPER_R)
						})?;

				assert_eq!(InputState::Pressed { just_now: false }, state);
				Ok(())
			}

			#[test]
			fn use_correct_user_input() -> Result<(), RunSystemError> {
				let mut app = setup(
					_Map::new().with_mock(|mock| {
						mock.expect_get_input::<PlayerSlot>()
							.return_const(UserInput::KeyCode(KeyCode::Enter));
					}),
					_UserInput::new().with_mock(|mock| {
						mock.expect_get_user_input_state()
							.times(1)
							.with(eq(UserInput::KeyCode(KeyCode::Enter)))
							.return_const(InputState::Pressed { just_now: false });
					}),
				);

				app.world_mut()
					.run_system_once(|input: InputMut<_Map, _UserInput>| {
						input.get_action_input_state(PlayerSlot::UPPER_R);
					})
			}
		}
	}
}
