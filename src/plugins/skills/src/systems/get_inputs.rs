use crate::{items::slot_key::SlotKey, traits::InputState};
use bevy::{
	ecs::system::{Res, Resource},
	input::keyboard::KeyCode,
};
use common::traits::map_value::TryMapBackwards;

#[derive(Default, Debug, PartialEq, Clone)]
pub(crate) struct Input {
	pub just_pressed: Vec<SlotKey>,
	pub pressed: Vec<SlotKey>,
	pub just_released: Vec<SlotKey>,
}

pub(crate) fn get_inputs<
	TMap: Resource + TryMapBackwards<KeyCode, SlotKey>,
	TSuperiorInput: Resource + InputState<TMap, KeyCode>,
	TInferiorInput: Resource + InputState<TMap, KeyCode>,
>(
	key_map: Res<TMap>,
	superior: Res<TSuperiorInput>,
	inferior: Res<TInferiorInput>,
) -> Input {
	let mut just_pressed = superior.just_pressed_slots(&key_map);
	let mut pressed = superior.pressed_slots(&key_map);
	let mut just_released = superior.just_released_slots(&key_map);

	pressed.extend(inferior.pressed_slots(&key_map));
	just_pressed.extend(
		inferior
			.just_pressed_slots(&key_map)
			.iter()
			.filter(|key| !pressed.contains(key)),
	);
	just_released.extend(
		inferior
			.just_released_slots(&key_map)
			.iter()
			.filter(|key| !pressed.contains(key)),
	);

	Input {
		just_pressed,
		pressed,
		just_released,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::{In, IntoSystem, ResMut, Resource},
		input::keyboard::KeyCode,
		utils::default,
	};
	use common::{
		components::Side,
		test_tools::utils::SingleThreadedApp,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::automock;

	#[derive(Resource, Clone, Debug, PartialEq)]
	struct _Map;

	impl TryMapBackwards<KeyCode, SlotKey> for _Map {
		fn try_map_backwards(&self, _: KeyCode) -> Option<SlotKey> {
			None
		}
	}

	#[derive(Resource, Debug, PartialEq, Default)]
	struct _Result(Input);

	#[derive(Resource, NestedMocks)]
	struct _Superior {
		mock: Mock_Superior,
	}

	#[automock]
	impl InputState<_Map, KeyCode> for _Superior {
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

	#[derive(Resource, NestedMocks)]
	struct _Inferior {
		mock: Mock_Inferior,
	}

	#[automock]
	impl InputState<_Map, KeyCode> for _Inferior {
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

	fn setup(superior: _Superior, inferior: _Inferior, map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Result>();
		app.insert_resource(map);
		app.insert_resource(superior);
		app.insert_resource(inferior);
		app.add_systems(
			Update,
			get_inputs::<_Map, _Superior, _Inferior>
				.pipe(|input: In<Input>, mut result: ResMut<_Result>| result.0 = input.0),
		);

		app
	}

	#[test]
	fn get_superior_inputs() {
		let mut app = setup(
			_Superior::new().with_mock(|mock| {
				mock.expect_just_pressed_slots()
					.times(1)
					.return_const(vec![SlotKey::Hand(Side::Right)]);
				mock.expect_pressed_slots()
					.times(1)
					.return_const(vec![SlotKey::Hand(Side::Right)]);
				mock.expect_just_released_slots()
					.times(1)
					.return_const(vec![SlotKey::Hand(Side::Left)]);
			}),
			_Inferior::new().with_mock(|mock| {
				mock.expect_just_pressed_slots().return_const(vec![]);
				mock.expect_pressed_slots().return_const(vec![]);
				mock.expect_just_released_slots().return_const(vec![]);
			}),
			_Map,
		);

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(
			&_Result(Input {
				just_pressed: vec![SlotKey::Hand(Side::Right)],
				pressed: vec![SlotKey::Hand(Side::Right)],
				just_released: vec![SlotKey::Hand(Side::Left)],
			}),
			result
		);
	}

	#[test]
	fn get_inferior_inputs() {
		let mut app = setup(
			_Superior::new().with_mock(|mock| {
				mock.expect_just_pressed_slots().return_const(vec![]);
				mock.expect_pressed_slots().return_const(vec![]);
				mock.expect_just_released_slots().return_const(vec![]);
			}),
			_Inferior::new().with_mock(|mock| {
				mock.expect_just_pressed_slots()
					.times(1)
					.return_const(vec![SlotKey::Hand(Side::Left)]);
				mock.expect_pressed_slots()
					.times(1)
					.return_const(vec![SlotKey::Hand(Side::Right)]);
				mock.expect_just_released_slots()
					.times(1)
					.return_const(vec![SlotKey::Hand(Side::Left)]);
			}),
			_Map,
		);

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(
			&_Result(Input {
				just_pressed: vec![SlotKey::Hand(Side::Left)],
				pressed: vec![SlotKey::Hand(Side::Right)],
				just_released: vec![SlotKey::Hand(Side::Left)],
			}),
			result
		);
	}

	#[test]
	fn ignore_inferior_just_pressed_if_superior_pressed() {
		let mut app = setup(
			_Superior::new().with_mock(|mock| {
				mock.expect_just_pressed_slots().return_const(vec![]);
				mock.expect_pressed_slots()
					.return_const(vec![SlotKey::Hand(Side::Right)]);
				mock.expect_just_released_slots().return_const(vec![]);
			}),
			_Inferior::new().with_mock(|mock| {
				mock.expect_just_pressed_slots()
					.return_const(vec![SlotKey::Hand(Side::Right)]);
				mock.expect_pressed_slots().return_const(vec![]);
				mock.expect_just_released_slots().return_const(vec![]);
			}),
			_Map,
		);

		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(
			&_Result(Input {
				pressed: vec![SlotKey::Hand(Side::Right)],
				..default()
			}),
			result
		);
	}

	#[test]
	fn ignore_inferior_just_released_if_superior_pressed() {
		let mut app = setup(
			_Superior::new().with_mock(|mock| {
				mock.expect_just_pressed_slots().return_const(vec![]);
				mock.expect_pressed_slots()
					.return_const(vec![SlotKey::Hand(Side::Right)]);
				mock.expect_just_released_slots().return_const(vec![]);
			}),
			_Inferior::new().with_mock(|mock| {
				mock.expect_just_pressed_slots().return_const(vec![]);
				mock.expect_pressed_slots().return_const(vec![]);
				mock.expect_just_released_slots()
					.return_const(vec![SlotKey::Hand(Side::Right)]);
			}),
			_Map,
		);
		app.update();

		let result = app.world().resource::<_Result>();

		assert_eq!(
			&_Result(Input {
				pressed: vec![SlotKey::Hand(Side::Right)],
				..default()
			}),
			result
		);
	}
}
