use crate::{items::SlotKey, resources::SlotMap, traits::InputState};
use bevy::{
	ecs::system::{Res, Resource},
	input::keyboard::KeyCode,
};

#[derive(Default, Debug, PartialEq, Clone)]
pub(crate) struct Input {
	pub just_pressed: Vec<SlotKey>,
	pub pressed: Vec<SlotKey>,
	pub just_released: Vec<SlotKey>,
}

pub(crate) fn get_inputs<
	TSuperiorInput: Resource + InputState<KeyCode>,
	TInferiorInput: Resource + InputState<KeyCode>,
>(
	slot_map: Res<SlotMap>,
	superior: Res<TSuperiorInput>,
	inferior: Res<TInferiorInput>,
) -> Input {
	let mut just_pressed = superior.just_pressed_slots(&slot_map);
	let mut pressed = superior.pressed_slots(&slot_map);
	let mut just_released = superior.just_released_slots(&slot_map);

	pressed.extend(inferior.pressed_slots(&slot_map));
	just_pressed.extend(
		inferior
			.just_pressed_slots(&slot_map)
			.iter()
			.filter(|key| !pressed.contains(key)),
	);
	just_released.extend(
		inferior
			.just_released_slots(&slot_map)
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
	use crate::resources::SlotMap;
	use bevy::{
		app::{App, Update},
		ecs::system::{In, IntoSystem, ResMut, Resource},
		input::keyboard::KeyCode,
		utils::default,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use mockall::{automock, predicate::eq};

	#[derive(Resource, Debug, PartialEq, Default)]
	struct _Result(Input);

	#[derive(Resource, Default)]
	struct _Superior {
		mock: Mock_Superior,
	}

	#[automock]
	impl InputState<KeyCode> for _Superior {
		fn just_pressed_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
			self.mock.just_pressed_slots(map)
		}
		fn pressed_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
			self.mock.pressed_slots(map)
		}
		fn just_released_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
			self.mock.just_released_slots(map)
		}
	}

	#[derive(Resource, Default)]
	struct _Inferior {
		mock: Mock_Inferior,
	}

	#[automock]
	impl InputState<KeyCode> for _Inferior {
		fn just_pressed_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
			self.mock.just_pressed_slots(map)
		}
		fn pressed_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
			self.mock.pressed_slots(map)
		}
		fn just_released_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
			self.mock.just_released_slots(map)
		}
	}

	fn setup(superior: _Superior, inferior: _Inferior) -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Result>();
		app.init_resource::<SlotMap>();
		app.insert_resource(superior);
		app.insert_resource(inferior);
		app.add_systems(
			Update,
			get_inputs::<_Superior, _Inferior>
				.pipe(|input: In<Input>, mut result: ResMut<_Result>| result.0 = input.0),
		);

		app
	}

	#[test]
	fn get_superior_inputs() {
		let slot_map = SlotMap::new([(KeyCode::KeyA, SlotKey::SkillSpawn, "")]);
		let mut sup = _Superior::default();
		sup.mock
			.expect_just_pressed_slots()
			.times(1)
			.with(eq(slot_map.clone()))
			.return_const(vec![SlotKey::Hand(Side::Main)]);
		sup.mock
			.expect_pressed_slots()
			.times(1)
			.with(eq(slot_map.clone()))
			.return_const(vec![SlotKey::Hand(Side::Off)]);
		sup.mock
			.expect_just_released_slots()
			.times(1)
			.with(eq(slot_map.clone()))
			.return_const(vec![SlotKey::SkillSpawn]);

		let mut inf = _Inferior::default();
		inf.mock.expect_just_pressed_slots().return_const(vec![]);
		inf.mock.expect_pressed_slots().return_const(vec![]);
		inf.mock.expect_just_released_slots().return_const(vec![]);

		let mut app = setup(sup, inf);
		app.insert_resource(slot_map);
		app.update();

		let result = app.world.resource::<_Result>();

		assert_eq!(
			&_Result(Input {
				just_pressed: vec![SlotKey::Hand(Side::Main)],
				pressed: vec![SlotKey::Hand(Side::Off)],
				just_released: vec![SlotKey::SkillSpawn],
			}),
			result
		);
	}

	#[test]
	fn get_inferior_inputs() {
		let slot_map = SlotMap::new([(KeyCode::KeyA, SlotKey::SkillSpawn, "")]);
		let mut inf = _Inferior::default();
		inf.mock
			.expect_just_pressed_slots()
			.times(1)
			.with(eq(slot_map.clone()))
			.return_const(vec![SlotKey::Hand(Side::Main)]);
		inf.mock
			.expect_pressed_slots()
			.times(1)
			.with(eq(slot_map.clone()))
			.return_const(vec![SlotKey::Hand(Side::Off)]);
		inf.mock
			.expect_just_released_slots()
			.times(1)
			.with(eq(slot_map.clone()))
			.return_const(vec![SlotKey::SkillSpawn]);

		let mut sup = _Superior::default();
		sup.mock.expect_just_pressed_slots().return_const(vec![]);
		sup.mock.expect_pressed_slots().return_const(vec![]);
		sup.mock.expect_just_released_slots().return_const(vec![]);

		let mut app = setup(sup, inf);
		app.insert_resource(slot_map);
		app.update();

		let result = app.world.resource::<_Result>();

		assert_eq!(
			&_Result(Input {
				just_pressed: vec![SlotKey::Hand(Side::Main)],
				pressed: vec![SlotKey::Hand(Side::Off)],
				just_released: vec![SlotKey::SkillSpawn],
			}),
			result
		);
	}

	#[test]
	fn ignore_inferior_just_pressed_if_superior_pressed() {
		let mut sup = _Superior::default();
		sup.mock.expect_just_pressed_slots().return_const(vec![]);
		sup.mock
			.expect_pressed_slots()
			.return_const(vec![SlotKey::Hand(Side::Main)]);
		sup.mock.expect_just_released_slots().return_const(vec![]);

		let mut inf = _Inferior::default();
		inf.mock
			.expect_just_pressed_slots()
			.return_const(vec![SlotKey::Hand(Side::Main)]);
		inf.mock.expect_pressed_slots().return_const(vec![]);
		inf.mock.expect_just_released_slots().return_const(vec![]);

		let mut app = setup(sup, inf);
		app.update();

		let result = app.world.resource::<_Result>();

		assert_eq!(
			&_Result(Input {
				pressed: vec![SlotKey::Hand(Side::Main)],
				..default()
			}),
			result
		);
	}

	#[test]
	fn ignore_inferior_just_released_if_superior_pressed() {
		let mut sup = _Superior::default();
		sup.mock.expect_just_pressed_slots().return_const(vec![]);
		sup.mock
			.expect_pressed_slots()
			.return_const(vec![SlotKey::Hand(Side::Main)]);
		sup.mock.expect_just_released_slots().return_const(vec![]);

		let mut inf = _Inferior::default();
		inf.mock.expect_just_pressed_slots().return_const(vec![]);
		inf.mock.expect_pressed_slots().return_const(vec![]);
		inf.mock
			.expect_just_released_slots()
			.return_const(vec![SlotKey::Hand(Side::Main)]);

		let mut app = setup(sup, inf);
		app.update();

		let result = app.world.resource::<_Result>();

		assert_eq!(
			&_Result(Input {
				pressed: vec![SlotKey::Hand(Side::Main)],
				..default()
			}),
			result
		);
	}
}
