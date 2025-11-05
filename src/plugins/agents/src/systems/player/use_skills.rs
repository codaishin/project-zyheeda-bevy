use crate::components::player::Player;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::slot::PlayerSlot,
	traits::{
		accessors::get::EntityContextMut,
		handles_input::{GetAllInputStates, InputState},
		handles_skills_control::{HoldSkill, SkillControl},
	},
};

impl Player {
	pub(crate) fn use_skills<TInput, TSkills>(
		mut skills: StaticSystemParam<TSkills>,
		input: StaticSystemParam<TInput>,
		players: Query<Entity, With<Self>>,
	) where
		TInput: for<'w, 's> SystemParam<Item<'w, 's>: GetAllInputStates>,
		TSkills: for<'c> EntityContextMut<SkillControl, TContext<'c>: HoldSkill>,
	{
		let held = || {
			input
				.get_all_input_states::<PlayerSlot>()
				.filter_map(|(key, state)| match state {
					InputState::Pressed { .. } => Some(key),
					_ => None,
				})
		};

		for entity in &players {
			let ctx = TSkills::get_entity_context_mut(&mut skills, entity, SkillControl);
			let Some(mut ctx) = ctx else {
				continue;
			};

			for key in held() {
				ctx.holding(key);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::action_key::{
			ActionKey,
			slot::{PlayerSlot, SlotKey},
		},
		traits::{handles_input::InputState, iteration::IterFinite},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp};

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

	#[derive(Component, NestedMocks)]
	struct _Skills {
		mock: Mock_Skills,
	}

	#[automock]
	impl HoldSkill for _Skills {
		fn holding<TSlot>(&mut self, key: TSlot)
		where
			TSlot: Into<SlotKey> + 'static,
		{
			self.mock.holding(key);
		}
	}

	fn setup(input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input);
		app.add_systems(
			Update,
			Player::use_skills::<Res<_Input>, Query<&mut _Skills>>,
		);

		app
	}

	#[test_case(InputState::just_pressed(); "on just pressed")]
	#[test_case(InputState::pressed(); "on pressed")]
	fn hold_skill(state: InputState) {
		let mut app = setup(_Input::new().with_mock(move |mock| {
			mock.expect_get_all_input_states::<PlayerSlot>()
				.returning(move || Box::new(std::iter::once((PlayerSlot::UPPER_L, state))));
		}));
		app.world_mut().spawn((
			Player,
			_Skills::new().with_mock(|mock| {
				mock.expect_holding()
					.once()
					.with(eq(PlayerSlot::UPPER_L))
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn do_nothing_if_player_missing() {
		let mut app = setup(_Input::new().with_mock(move |mock| {
			mock.expect_get_all_input_states::<PlayerSlot>()
				.returning(move || {
					Box::new(std::iter::once((
						PlayerSlot::UPPER_L,
						InputState::pressed(),
					)))
				});
		}));
		app.world_mut().spawn(_Skills::new().with_mock(|mock| {
			mock.expect_holding::<PlayerSlot>().never();
		}));

		app.update();
	}
}
