use crate::components::skill_usage::SkillUsage;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::slot::{PlayerSlot, SlotKey},
	traits::handles_input::{GetAllInputStates, InputState},
};
use std::collections::HashSet;

impl SkillUsage {
	pub(crate) fn player<TPlayer, TInput>(
		input: StaticSystemParam<TInput>,
		mut players: Query<&mut SkillUsage, With<TPlayer>>,
	) where
		TPlayer: Component,
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputStates>,
	{
		if players.is_empty() {
			return;
		}

		let write = |mut skill_usage: Mut<SkillUsage>| {
			let mut started_holding = HashSet::default();
			let mut holding = HashSet::default();

			for (slot, state) in input.get_all_input_states::<PlayerSlot>() {
				match state {
					InputState::Pressed { just_now: true } => {
						started_holding.insert(SlotKey::from(slot));
						holding.insert(SlotKey::from(slot));
					}
					InputState::Pressed { just_now: false } => {
						holding.insert(SlotKey::from(slot));
					}
					_ => {}
				}
			}

			skill_usage.started_holding = started_holding;
			skill_usage.holding = holding;
		};

		for skill_usage in &mut players {
			write(skill_usage);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::action_key::{ActionKey, slot::SlotKey},
		traits::iteration::IterFinite,
	};
	use macros::NestedMocks;
	use mockall::automock;
	use std::collections::HashSet;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(SystemParam)]
	struct _InputParam<'w> {
		input: Res<'w, _Input>,
	}

	impl GetAllInputStates for _InputParam<'_> {
		fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
		where
			TAction: Into<ActionKey> + IterFinite + 'static,
		{
			self.input.get_all_input_states()
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl GetAllInputStates for _Input {
		fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
		where
			TAction: Into<ActionKey> + IterFinite + 'static,
		{
			self.mock.get_all_input_states()
		}
	}

	#[derive(Component)]
	#[require(SkillUsage)]
	struct _Player;

	fn setup(input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input);
		app.add_systems(Update, SkillUsage::player::<_Player, _InputParam>);

		app
	}

	#[test]
	fn set_just_pressed() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					PlayerSlot::LOWER_R,
					InputState::just_pressed(),
				)))
			});
		}));
		let entity = app.world_mut().spawn(_Player).id();

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
	fn set_pressed() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(
					[
						(PlayerSlot::LOWER_R, InputState::pressed()),
						(PlayerSlot::LOWER_L, InputState::just_pressed()),
					]
					.into_iter(),
				)
			});
		}));
		let entity = app.world_mut().spawn(_Player).id();

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
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(
					[
						(PlayerSlot::LOWER_R, InputState::pressed()),
						(PlayerSlot::LOWER_L, InputState::just_pressed()),
					]
					.into_iter(),
				)
			});
		}));
		let entity = app.world_mut().spawn(_Player).id();

		app.update();
		app.world_mut()
			.insert_resource(_Input::new().with_mock(|mock| {
				mock.expect_get_all_input_states().returning(|| {
					Box::new(
						[
							(PlayerSlot::UPPER_R, InputState::pressed()),
							(PlayerSlot::UPPER_L, InputState::just_pressed()),
						]
						.into_iter(),
					)
				});
			}));
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
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					PlayerSlot::LOWER_R,
					InputState::just_pressed(),
				)))
			});
		}));
		let entity = app.world_mut().spawn(SkillUsage::default()).id();

		app.update();

		assert_eq!(
			Some(&SkillUsage::default()),
			app.world().entity(entity).get::<SkillUsage>()
		);
	}
}
