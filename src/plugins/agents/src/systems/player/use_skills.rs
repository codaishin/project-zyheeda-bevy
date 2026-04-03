use crate::components::player::Player;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::slot::{HandSlot, SlotKey},
	traits::{
		accessors::get::GetContextMut,
		handles_input::{GetAllInputStates, InputState},
		handles_loadout::{CurrentTargetMut, HeldSkills, HeldSkillsMut, skills::Skills},
		handles_skill_physics::SkillTarget,
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
				.get_all_input_states::<HandSlot>()
				.filter_map(|(key, state)| match state {
					InputState::Pressed { .. } => Some(key),
					_ => None,
				})
		};

		for entity in &players {
			let Some(mut ctx) = TLoadout::get_context_mut(&mut skills, Skills { entity }) else {
				continue;
			};

			let new_held_skills = held().map(SlotKey::from).collect();

			if ctx.held_skills() == &new_held_skills {
				continue;
			}

			*ctx.current_target_mut() = Some(SkillTarget::Cursor);
			*ctx.held_skills_mut() = new_held_skills;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::action_key::{
			ActionKey,
			slot::{HandSlot, SlotKey},
		},
		traits::{
			handles_input::InputState,
			handles_loadout::{CurrentTarget, CurrentTargetMut, HeldSkills},
			handles_skill_physics::SkillTarget,
			iteration::IterFinite,
		},
	};
	use mockall::automock;
	use std::collections::{HashMap, HashSet};
	use test_case::test_case;
	use testing::{IsChanged, SingleThreadedApp};

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
	struct _Loadout {
		slots: HashSet<SlotKey>,
		target: Option<SkillTarget>,
	}

	impl _Loadout {
		fn with_target(mut self, target: Option<SkillTarget>) -> Self {
			self.target = target;
			self
		}
	}

	impl<const N: usize> From<[SlotKey; N]> for _Loadout {
		fn from(slots: [SlotKey; N]) -> Self {
			Self {
				slots: HashSet::from(slots),
				target: None,
			}
		}
	}

	impl HeldSkills for _Loadout {
		fn held_skills(&self) -> &HashSet<SlotKey> {
			&self.slots
		}
	}

	impl HeldSkillsMut for _Loadout {
		fn held_skills_mut(&mut self) -> &mut HashSet<SlotKey> {
			&mut self.slots
		}
	}

	impl CurrentTarget for _Loadout {
		fn current_target(&self) -> Option<&SkillTarget> {
			self.target.as_ref()
		}
	}

	impl CurrentTargetMut for _Loadout {
		fn current_target_mut(&mut self) -> &mut Option<SkillTarget> {
			&mut self.target
		}
	}

	fn setup(input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input);
		app.add_systems(
			Update,
			(
				Player::use_skills::<Res<_Input>, Query<&mut _Loadout>>,
				IsChanged::<_Loadout>::detect,
			)
				.chain(),
		);

		app
	}

	#[test_case(InputState::just_pressed(); "on just pressed")]
	#[test_case(InputState::pressed(); "on pressed")]
	fn set_held_skills(state: InputState) {
		let mut app = setup(_Input::from(std::iter::once((HandSlot::Left, state))));
		let entity = app.world_mut().spawn((Player, _Loadout::default())).id();

		app.update();

		assert_eq!(
			Some(
				&_Loadout::from([SlotKey::from(HandSlot::Left)])
					.with_target(Some(SkillTarget::Cursor))
			),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test_case(InputState::just_pressed(); "on just pressed")]
	#[test_case(InputState::pressed(); "on pressed")]
	fn override_held_skills(state: InputState) {
		let mut app = setup(_Input::from(std::iter::once((HandSlot::Left, state))));
		let entity = app
			.world_mut()
			.spawn((Player, _Loadout::from([SlotKey::from(HandSlot::Right)])))
			.id();

		app.update();

		assert_eq!(
			Some(
				&_Loadout::from([SlotKey::from(HandSlot::Left)])
					.with_target(Some(SkillTarget::Cursor))
			),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test]
	fn do_nothing_if_player_missing() {
		let mut app = setup(_Input::from(std::iter::once((
			HandSlot::Left,
			InputState::pressed(),
		))));
		let entity = app.world_mut().spawn(_Loadout::default()).id();

		app.update();

		assert_eq!(
			Some(&_Loadout::default()),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test_case(InputState::just_pressed(); "on just pressed")]
	#[test_case(InputState::pressed(); "on pressed")]
	fn do_nothing_if_current_held_would_not_change(state: InputState) {
		let mut app = setup(_Input::from(std::iter::once((HandSlot::Left, state))));
		let entity = app
			.world_mut()
			.spawn((Player, _Loadout::from([SlotKey::from(HandSlot::Left)])))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<_Loadout>>(),
		);
	}
}
