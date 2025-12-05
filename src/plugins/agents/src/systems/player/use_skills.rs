use crate::components::player::Player;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::slot::{PlayerSlot, SlotKey},
	traits::{
		accessors::get::GetContextMut,
		handles_input::{GetAllInputStates, InputState},
		handles_loadout::{HeldSkillsMut, skills::Skills},
	},
};

impl Player {
	pub(crate) fn use_skills<TInput, TLoadout>(
		mut skills: StaticSystemParam<TLoadout>,
		input: StaticSystemParam<TInput>,
		players: Query<Entity, With<Self>>,
	) where
		TInput: for<'w, 's> SystemParam<Item<'w, 's>: GetAllInputStates>,
		TLoadout: for<'c> GetContextMut<Skills, TContext<'c>: HeldSkillsMut>,
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
			let Some(mut ctx) = TLoadout::get_context_mut(&mut skills, Skills { entity }) else {
				continue;
			};

			*ctx.held_skills_mut() = held().map(SlotKey::from).collect();
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
		traits::{handles_input::InputState, handles_loadout::HeldSkills, iteration::IterFinite},
	};
	use mockall::automock;
	use std::collections::{HashMap, HashSet};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Resource)]
	struct _Input(HashMap<ActionKey, InputState>);

	impl<TAction, T> From<T> for _Input
	where
		TAction: TryInto<ActionKey>,
		T: IntoIterator<Item = (TAction, InputState)>,
	{
		fn from(inputs: T) -> Self {
			Self(
				inputs
					.into_iter()
					.filter_map(|(a, i)| Some((a.try_into().ok()?, i)))
					.collect(),
			)
		}
	}

	#[automock]
	impl GetAllInputStates for _Input {
		fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
		where
			TAction: Into<ActionKey> + IterFinite + 'static,
		{
			TAction::iterator().filter_map(|a| Some((a, self.0.get(&a.into()).copied()?)))
		}
	}

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Loadout(HashSet<SlotKey>);

	impl<const N: usize> From<[SlotKey; N]> for _Loadout {
		fn from(slots: [SlotKey; N]) -> Self {
			Self(HashSet::from(slots))
		}
	}

	impl HeldSkills for _Loadout {
		fn held_skills(&self) -> &HashSet<SlotKey> {
			&self.0
		}
	}

	impl HeldSkillsMut for _Loadout {
		fn held_skills_mut(&mut self) -> &mut HashSet<SlotKey> {
			&mut self.0
		}
	}

	fn setup(input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input);
		app.add_systems(
			Update,
			Player::use_skills::<Res<_Input>, Query<&mut _Loadout>>,
		);

		app
	}

	#[test_case(InputState::just_pressed(); "on just pressed")]
	#[test_case(InputState::pressed(); "on pressed")]
	fn insert_held_skill(state: InputState) {
		let mut app = setup(_Input::from(std::iter::once((PlayerSlot::UPPER_L, state))));
		let entity = app.world_mut().spawn((Player, _Loadout::default())).id();

		app.update();

		assert_eq!(
			Some(&_Loadout::from([SlotKey::from(PlayerSlot::UPPER_L)])),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test_case(InputState::just_pressed(); "on just pressed")]
	#[test_case(InputState::pressed(); "on pressed")]
	fn override_held_skills(state: InputState) {
		let mut app = setup(_Input::from(std::iter::once((PlayerSlot::UPPER_L, state))));
		let entity = app
			.world_mut()
			.spawn((Player, _Loadout::from([SlotKey::from(PlayerSlot::UPPER_R)])))
			.id();

		app.update();

		assert_eq!(
			Some(&_Loadout::from([SlotKey::from(PlayerSlot::UPPER_L)])),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test]
	fn do_nothing_if_player_missing() {
		let mut app = setup(_Input::from(std::iter::once((
			PlayerSlot::UPPER_L,
			InputState::pressed(),
		))));
		let entity = app.world_mut().spawn(_Loadout::default()).id();

		app.update();

		assert_eq!(
			Some(&_Loadout::default()),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}
}
