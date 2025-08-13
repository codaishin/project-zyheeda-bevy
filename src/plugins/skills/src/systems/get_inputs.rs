use crate::traits::InputState;
use bevy::prelude::*;
use common::{
	tools::action_key::{slot::PlayerSlot, user_input::UserInput},
	traits::key_mappings::TryGetAction,
};

#[derive(Default, Debug, PartialEq, Clone)]
pub(crate) struct Input {
	pub just_pressed: Vec<PlayerSlot>,
	pub pressed: Vec<PlayerSlot>,
}

pub(crate) fn get_inputs<
	TMap: Resource + TryGetAction<PlayerSlot, TInput = UserInput>,
	TInput: Resource + InputState<TMap, PlayerSlot>,
>(
	key_map: Res<TMap>,
	input: Res<TInput>,
) -> Input {
	Input {
		just_pressed: input.just_pressed_slots(&key_map),
		pressed: input.pressed_slots(&key_map),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::action_key::slot::Side;
	use macros::NestedMocks;
	use mockall::automock;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, Clone, Debug, PartialEq)]
	struct _Map;

	impl TryGetAction<PlayerSlot> for _Map {
		type TInput = UserInput;

		fn try_get_action(&self, _: UserInput) -> Option<PlayerSlot> {
			None
		}
	}

	#[derive(Resource, Debug, PartialEq, Default)]
	struct _Result(Input);

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl InputState<_Map, PlayerSlot> for _Input {
		fn just_pressed_slots(&self, map: &_Map) -> Vec<PlayerSlot> {
			self.mock.just_pressed_slots(map)
		}
		fn pressed_slots(&self, map: &_Map) -> Vec<PlayerSlot> {
			self.mock.pressed_slots(map)
		}
	}

	fn setup(input: _Input, map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Result>();
		app.insert_resource(map);
		app.insert_resource(input);
		app.add_systems(
			Update,
			get_inputs::<_Map, _Input>
				.pipe(|input: In<Input>, mut result: ResMut<_Result>| result.0 = input.0),
		);

		app
	}

	#[test]
	fn return_inputs() {
		let mut app = setup(
			_Input::new().with_mock(|mock| {
				mock.expect_just_pressed_slots()
					.times(1)
					.return_const(vec![PlayerSlot::Lower(Side::Right)]);
				mock.expect_pressed_slots()
					.times(1)
					.return_const(vec![PlayerSlot::Lower(Side::Left)]);
			}),
			_Map,
		);

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(
			&_Result(Input {
				just_pressed: vec![PlayerSlot::Lower(Side::Right)],
				pressed: vec![PlayerSlot::Lower(Side::Left)],
			}),
			result
		);
	}
}
