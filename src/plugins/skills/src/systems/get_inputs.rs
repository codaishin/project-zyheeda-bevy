use crate::traits::InputState;
use bevy::prelude::*;
use common::{
	tools::action_key::{slot::SlotKey, user_input::UserInput},
	traits::key_mappings::TryGetAction,
};

#[derive(Default, Debug, PartialEq, Clone)]
pub(crate) struct Input {
	pub just_pressed: Vec<SlotKey>,
	pub pressed: Vec<SlotKey>,
	pub just_released: Vec<SlotKey>,
}

pub(crate) fn get_inputs<
	TMap: Resource + TryGetAction<UserInput, SlotKey>,
	TInput: Resource + InputState<TMap, UserInput>,
>(
	key_map: Res<TMap>,
	input: Res<TInput>,
) -> Input {
	Input {
		just_pressed: input.just_pressed_slots(&key_map),
		pressed: input.pressed_slots(&key_map),
		just_released: input.just_released_slots(&key_map),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::action_key::slot::Side,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::automock;

	#[derive(Resource, Clone, Debug, PartialEq)]
	struct _Map;

	impl TryGetAction<UserInput, SlotKey> for _Map {
		fn try_get_action(&self, _: UserInput) -> Option<SlotKey> {
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
	impl InputState<_Map, UserInput> for _Input {
		fn just_pressed_slots(&self, map: &_Map) -> Vec<SlotKey> {
			self.mock.just_pressed_slots(map)
		}
		fn pressed_slots(&self, map: &_Map) -> Vec<SlotKey> {
			self.mock.pressed_slots(map)
		}
		fn just_released_slots(&self, map: &_Map) -> Vec<SlotKey> {
			self.mock.just_released_slots(map)
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
					.return_const(vec![SlotKey::BottomHand(Side::Right)]);
				mock.expect_pressed_slots()
					.times(1)
					.return_const(vec![SlotKey::BottomHand(Side::Right)]);
				mock.expect_just_released_slots()
					.times(1)
					.return_const(vec![SlotKey::BottomHand(Side::Left)]);
			}),
			_Map,
		);

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(
			&_Result(Input {
				just_pressed: vec![SlotKey::BottomHand(Side::Right)],
				pressed: vec![SlotKey::BottomHand(Side::Right)],
				just_released: vec![SlotKey::BottomHand(Side::Left)],
			}),
			result
		);
	}
}
