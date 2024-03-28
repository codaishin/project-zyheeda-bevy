use crate::{components::SlotKey, resources::SlotMap, traits::InputState};
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

pub(crate) fn get_input<TInput: InputState<KeyCode> + Resource>(
	input: Res<TInput>,
	slot_map: Res<SlotMap<KeyCode>>,
) -> Input {
	Input {
		just_pressed: input.just_pressed_slots(&slot_map),
		pressed: input.pressed_slots(&slot_map),
		just_released: input.just_released_slots(&slot_map),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::{Commands, In, IntoSystem, Resource},
		input::{keyboard::KeyCode, ButtonInput},
		utils::default,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use mockall::{automock, predicate::eq};

	#[derive(Resource, Default)]
	struct _Result(Input);

	#[derive(Default, Resource)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl InputState<KeyCode> for _Input {
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

	fn setup(mock_input: _Input) -> App {
		let mut app = App::new_single_threaded([Update]);
		app.insert_resource(mock_input);
		app.insert_resource(SlotMap::<KeyCode>::default());
		app.init_resource::<ButtonInput<KeyCode>>();
		app.add_systems(
			Update,
			get_input::<_Input>.pipe(|input: In<Input>, mut commands: Commands| {
				commands.insert_resource(_Result(input.0))
			}),
		);
		app
	}

	#[test]
	fn get_just_pressed() {
		let mut input = _Input::default();
		input.mock.expect_pressed_slots().return_const(vec![]);
		input
			.mock
			.expect_just_pressed_slots()
			.return_const(vec![SlotKey::SkillSpawn, SlotKey::Hand(Side::Main)]);
		input.mock.expect_just_released_slots().return_const(vec![]);

		let mut app = setup(input);
		app.update();

		let input = &app.world.resource::<_Result>().0;

		assert_eq!(
			&Input {
				just_pressed: vec![SlotKey::SkillSpawn, SlotKey::Hand(Side::Main)],
				..default()
			},
			input
		);
	}

	#[test]
	fn get_just_pressed_called_with_slot_map() {
		let slot_map = SlotMap::new([(KeyCode::KeyC, SlotKey::SkillSpawn, "Key C")]);
		let mut input = _Input::default();
		input.mock.expect_pressed_slots().return_const(vec![]);
		input
			.mock
			.expect_just_pressed_slots()
			.times(1)
			.with(eq(slot_map.clone()))
			.return_const(vec![]);
		input.mock.expect_just_released_slots().return_const(vec![]);

		let mut app = setup(input);
		app.world.insert_resource(slot_map);
		app.update();
	}

	#[test]
	fn get_pressed() {
		let mut input = _Input::default();
		input
			.mock
			.expect_pressed_slots()
			.return_const(vec![SlotKey::SkillSpawn, SlotKey::Hand(Side::Main)]);
		input.mock.expect_just_pressed_slots().return_const(vec![]);
		input.mock.expect_just_released_slots().return_const(vec![]);

		let mut app = setup(input);
		app.update();

		let input = &app.world.resource::<_Result>().0;

		assert_eq!(
			&Input {
				pressed: vec![SlotKey::SkillSpawn, SlotKey::Hand(Side::Main)],
				..default()
			},
			input
		);
	}

	#[test]
	fn get_pressed_called_with_slot_map() {
		let slot_map = SlotMap::new([(KeyCode::KeyC, SlotKey::SkillSpawn, "Key C")]);
		let mut input = _Input::default();
		input
			.mock
			.expect_pressed_slots()
			.times(1)
			.with(eq(slot_map.clone()))
			.return_const(vec![]);
		input.mock.expect_just_pressed_slots().return_const(vec![]);
		input.mock.expect_just_released_slots().return_const(vec![]);

		let mut app = setup(input);
		app.world.insert_resource(slot_map);
		app.update();
	}

	#[test]
	fn get_just_released() {
		let mut input = _Input::default();
		input.mock.expect_pressed_slots().return_const(vec![]);
		input.mock.expect_just_pressed_slots().return_const(vec![]);
		input
			.mock
			.expect_just_released_slots()
			.return_const(vec![SlotKey::SkillSpawn, SlotKey::Hand(Side::Main)]);

		let mut app = setup(input);
		app.update();

		let input = &app.world.resource::<_Result>().0;

		assert_eq!(
			&Input {
				just_released: vec![SlotKey::SkillSpawn, SlotKey::Hand(Side::Main)],
				..default()
			},
			input
		);
	}

	#[test]
	fn get_just_released_called_with_slot_map() {
		let slot_map = SlotMap::new([(KeyCode::KeyC, SlotKey::SkillSpawn, "Key C")]);
		let mut input = _Input::default();
		input.mock.expect_pressed_slots().return_const(vec![]);
		input.mock.expect_just_pressed_slots().return_const(vec![]);
		input
			.mock
			.expect_just_released_slots()
			.times(1)
			.with(eq(slot_map.clone()))
			.return_const(vec![]);

		let mut app = setup(input);
		app.world.insert_resource(slot_map);
		app.update();
	}
}
