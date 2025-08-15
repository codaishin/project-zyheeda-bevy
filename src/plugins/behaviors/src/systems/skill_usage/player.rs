use std::collections::HashSet;

use crate::components::skill_usage::SkillUsage;
use bevy::prelude::*;
use common::{
	tools::action_key::{
		slot::{PlayerSlot, SlotKey},
		user_input::UserInput,
	},
	traits::key_mappings::TryGetAction,
};

impl SkillUsage {
	pub(crate) fn player<TPlayer, TMap>(
		input: Res<ButtonInput<UserInput>>,
		map: Res<TMap>,
		mut players: Query<&mut SkillUsage, With<TPlayer>>,
	) where
		TPlayer: Component,
		TMap: Resource + TryGetAction<PlayerSlot, TInput = UserInput>,
	{
		if players.is_empty() {
			return;
		}

		let just_pressed = || {
			input
				.get_just_pressed()
				.filter_map(|key| map.try_get_action(*key))
				.map(SlotKey::from)
		};
		let pressed = || {
			input
				.get_pressed()
				.filter_map(|key| map.try_get_action(*key))
				.map(SlotKey::from)
		};

		for mut skill_usage in &mut players {
			skill_usage.started_holding = HashSet::from_iter(just_pressed());
			skill_usage.holding = HashSet::from_iter(pressed());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::action_key::slot::SlotKey;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashSet;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl TryGetAction<PlayerSlot> for _Map {
		type TInput = UserInput;

		fn try_get_action(&self, input: UserInput) -> Option<PlayerSlot> {
			self.mock.try_get_action(input)
		}
	}

	#[derive(Component)]
	#[require(SkillUsage)]
	struct _Player;

	macro_rules! set_input {
		($app:expr, just_pressed($btn:expr)) => {
			let mut input = $app.world_mut().resource_mut::<ButtonInput<UserInput>>();
			input.press($btn);
		};
		($app:expr, pressed($btn:expr)) => {{
			let mut input = $app.world_mut().resource_mut::<ButtonInput<UserInput>>();
			input.press($btn);
			input.clear_just_pressed($btn);
		}};
		($app:expr, reset_all) => {{
			let mut input = $app.world_mut().resource_mut::<ButtonInput<UserInput>>();
			input.reset_all();
		}};
	}

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(map);
		app.init_resource::<ButtonInput<UserInput>>();
		app.add_systems(Update, SkillUsage::player::<_Player, _Map>);

		app
	}

	#[test]
	fn set_just_held() {
		let key_a = UserInput::KeyCode(KeyCode::KeyA);
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_try_get_action()
				.with(eq(key_a))
				.return_const(Some(PlayerSlot::LOWER_R));
		}));
		let entity = app.world_mut().spawn(_Player).id();

		set_input!(app, just_pressed(key_a));
		app.update();

		assert_eq!(
			Some(&SkillUsage {
				holding: HashSet::from([SlotKey::from(PlayerSlot::LOWER_R)]),
				started_holding: HashSet::from([SlotKey::from(PlayerSlot::LOWER_R)]),
			}),
			app.world().entity(entity).get::<SkillUsage>()
		);
	}

	#[test]
	fn just_held_vs_held() {
		let key_a = UserInput::KeyCode(KeyCode::KeyA);
		let key_b = UserInput::KeyCode(KeyCode::KeyB);
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_try_get_action()
				.with(eq(key_a))
				.return_const(Some(PlayerSlot::LOWER_R));
			mock.expect_try_get_action()
				.with(eq(key_b))
				.return_const(Some(PlayerSlot::LOWER_L));
		}));
		let entity = app.world_mut().spawn(_Player).id();

		set_input!(app, pressed(key_a));
		set_input!(app, just_pressed(key_b));
		app.update();

		assert_eq!(
			Some(&SkillUsage {
				holding: HashSet::from([
					SlotKey::from(PlayerSlot::LOWER_R),
					SlotKey::from(PlayerSlot::LOWER_L)
				]),
				started_holding: HashSet::from([SlotKey::from(PlayerSlot::LOWER_L)]),
			}),
			app.world().entity(entity).get::<SkillUsage>()
		);
	}

	#[test]
	fn override_previous_values() {
		let key_a = UserInput::KeyCode(KeyCode::KeyA);
		let key_b = UserInput::KeyCode(KeyCode::KeyB);
		let key_c = UserInput::KeyCode(KeyCode::KeyC);
		let key_d = UserInput::KeyCode(KeyCode::KeyD);
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_try_get_action()
				.with(eq(key_a))
				.return_const(Some(PlayerSlot::LOWER_R));
			mock.expect_try_get_action()
				.with(eq(key_b))
				.return_const(Some(PlayerSlot::LOWER_L));
			mock.expect_try_get_action()
				.with(eq(key_c))
				.return_const(Some(PlayerSlot::UPPER_R));
			mock.expect_try_get_action()
				.with(eq(key_d))
				.return_const(Some(PlayerSlot::UPPER_L));
		}));
		let entity = app.world_mut().spawn(_Player).id();

		set_input!(app, pressed(key_a));
		set_input!(app, just_pressed(key_b));
		app.update();
		set_input!(app, reset_all);
		set_input!(app, pressed(key_c));
		set_input!(app, just_pressed(key_d));
		app.update();

		assert_eq!(
			Some(&SkillUsage {
				holding: HashSet::from([
					SlotKey::from(PlayerSlot::UPPER_R),
					SlotKey::from(PlayerSlot::UPPER_L)
				]),
				started_holding: HashSet::from([SlotKey::from(PlayerSlot::UPPER_L)]),
			}),
			app.world().entity(entity).get::<SkillUsage>()
		);
	}

	#[test]
	fn ignore_when_player_missing() {
		let key_a = UserInput::KeyCode(KeyCode::KeyA);
		let mut app = setup(_Map::new().with_mock(|mock| {
			mock.expect_try_get_action()
				.with(eq(key_a))
				.return_const(Some(PlayerSlot::LOWER_R));
		}));
		let entity = app.world_mut().spawn(SkillUsage::default()).id();

		set_input!(app, just_pressed(key_a));
		app.update();

		assert_eq!(
			Some(&SkillUsage::default()),
			app.world().entity(entity).get::<SkillUsage>()
		);
	}
}
