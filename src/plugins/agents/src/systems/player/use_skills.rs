use crate::components::player::Player;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::{
		slot::{HandSlot, SlotKey},
		targeting::TerrainTargeting,
	},
	traits::{
		accessors::get::GetContextMut,
		handles_input::{GetAllInputStates, InputState},
		handles_loadout::{
			CurrentTarget,
			CurrentTargetMut,
			HeldSkills,
			HeldSkillsMut,
			skills::Skills,
		},
		handles_skill_physics::{Cursor, InitializedAgent, SkillTarget, Target, TargetMut},
	},
};

impl Player {
	pub(crate) fn use_skills<TInput, TPhysics, TLoadout>(
		mut loadout: StaticSystemParam<TLoadout>,
		mut physics: StaticSystemParam<TPhysics>,
		input: StaticSystemParam<TInput>,
		players: Query<Entity, With<Self>>,
	) where
		TInput: for<'w, 's> SystemParam<Item<'w, 's>: GetAllInputStates>,
		TPhysics: for<'c> GetContextMut<InitializedAgent, TContext<'c>: TargetMut>,
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
		let get_cursor = || {
			let target_terrain =
				input
					.get_all_input_states::<TerrainTargeting>()
					.any(|(key, state)| {
						matches!((key, state), (TerrainTargeting, InputState::Pressed { .. }))
					});

			if target_terrain {
				Cursor::TerrainHover
			} else {
				Cursor::Direction
			}
		};

		for entity in &players {
			let skill_target = SkillTarget::Cursor(get_cursor());

			let agent = InitializedAgent { entity };
			if let Some(mut ctx) = TPhysics::get_context_mut(&mut physics, agent)
				&& ctx.target() != Some(&skill_target)
			{
				*ctx.target_mut() = Some(skill_target);
			};

			let skills = Skills { entity };
			if let Some(mut ctx) = TLoadout::get_context_mut(&mut loadout, skills) {
				if ctx.current_target() != Some(&skill_target) {
					*ctx.current_target_mut() = Some(skill_target);
				}

				let new_held_skills = held().map(SlotKey::from).collect();
				if ctx.held_skills() != &new_held_skills {
					*ctx.held_skills_mut() = new_held_skills;
				}
			};
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
			handles_skill_physics::{SkillTarget, Target},
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
		TAction: Into<ActionKey>,
		T: IntoIterator<Item = (TAction, InputState)>,
	{
		fn from(inputs: T) -> Self {
			Self(inputs.into_iter().map(|(a, i)| (a.into(), i)).collect())
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
	struct _Physics {
		target: Option<SkillTarget>,
	}

	impl Target for _Physics {
		fn target(&self) -> Option<&SkillTarget> {
			self.target.as_ref()
		}
	}

	impl TargetMut for _Physics {
		fn target_mut(&mut self) -> &mut Option<SkillTarget> {
			&mut self.target
		}
	}

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Loadout {
		slots_dereferenced: bool,
		slots: HashSet<SlotKey>,
		target_dereferenced: bool,
		target: Option<SkillTarget>,
	}

	impl _Loadout {
		fn with_target(mut self, target: Option<SkillTarget>) -> Self {
			self.target = target;
			self
		}

		fn reset_change_states(loadout: Query<&mut Self>) {
			for mut loadout in loadout {
				loadout.slots_dereferenced = false;
				loadout.target_dereferenced = false;
			}
		}
	}

	impl<const N: usize> From<[SlotKey; N]> for _Loadout {
		fn from(slots: [SlotKey; N]) -> Self {
			Self {
				slots_dereferenced: true,
				slots: HashSet::from(slots),
				target_dereferenced: true,
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
			self.slots_dereferenced = true;
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
			self.target_dereferenced = true;
			&mut self.target
		}
	}

	fn setup(input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input);
		app.add_systems(
			Update,
			(
				_Loadout::reset_change_states,
				Player::use_skills::<Res<_Input>, Query<&mut _Physics>, Query<&mut _Loadout>>,
				IsChanged::<_Physics>::detect,
			)
				.chain(),
		);

		app
	}

	mod default_targeting {
		use super::*;
		use test_case::test_case;

		#[test_case(InputState::just_pressed(); "on just pressed")]
		#[test_case(InputState::pressed(); "on pressed")]
		fn set_held_skills(state: InputState) {
			let mut app = setup(_Input::from(std::iter::once((HandSlot::Left, state))));
			let entity = app.world_mut().spawn((Player, _Loadout::default())).id();

			app.update();

			assert_eq!(
				Some(
					&_Loadout::from([SlotKey::from(HandSlot::Left)])
						.with_target(Some(SkillTarget::Cursor(Cursor::Direction)))
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
						.with_target(Some(SkillTarget::Cursor(Cursor::Direction)))
				),
				app.world().entity(entity).get::<_Loadout>(),
			);
		}

		#[test_case(InputState::just_pressed(); "on just pressed")]
		#[test_case(InputState::pressed(); "on pressed")]
		fn set_skill_target(state: InputState) {
			let mut app = setup(_Input::from([(ActionKey::from(HandSlot::Left), state)]));
			let entity = app.world_mut().spawn((Player, _Physics::default())).id();

			app.update();

			assert_eq!(
				Some(&_Physics {
					target: Some(SkillTarget::Cursor(Cursor::Direction))
				}),
				app.world().entity(entity).get::<_Physics>(),
			);
		}
	}

	mod terrain_targeting {
		use super::*;
		use common::tools::action_key::targeting::TerrainTargeting;
		use test_case::test_case;

		#[test_case(InputState::just_pressed(); "on just pressed")]
		#[test_case(InputState::pressed(); "on pressed")]
		fn set_held_skills(state: InputState) {
			let mut app = setup(_Input::from([
				(ActionKey::from(HandSlot::Left), state),
				(ActionKey::from(TerrainTargeting), state),
			]));
			let entity = app.world_mut().spawn((Player, _Loadout::default())).id();

			app.update();

			assert_eq!(
				Some(
					&_Loadout::from([SlotKey::from(HandSlot::Left)])
						.with_target(Some(SkillTarget::Cursor(Cursor::TerrainHover)))
				),
				app.world().entity(entity).get::<_Loadout>(),
			);
		}

		#[test_case(InputState::just_pressed(); "on just pressed")]
		#[test_case(InputState::pressed(); "on pressed")]
		fn override_held_skills(state: InputState) {
			let mut app = setup(_Input::from([
				(ActionKey::from(HandSlot::Left), state),
				(ActionKey::from(TerrainTargeting), state),
			]));
			let entity = app
				.world_mut()
				.spawn((Player, _Loadout::from([SlotKey::from(HandSlot::Right)])))
				.id();

			app.update();

			assert_eq!(
				Some(
					&_Loadout::from([SlotKey::from(HandSlot::Left)])
						.with_target(Some(SkillTarget::Cursor(Cursor::TerrainHover)))
				),
				app.world().entity(entity).get::<_Loadout>(),
			);
		}

		#[test_case(InputState::just_pressed(); "on just pressed")]
		#[test_case(InputState::pressed(); "on pressed")]
		fn set_skill_target(state: InputState) {
			let mut app = setup(_Input::from([
				(ActionKey::from(HandSlot::Left), state),
				(ActionKey::from(TerrainTargeting), state),
			]));
			let entity = app.world_mut().spawn((Player, _Physics::default())).id();

			app.update();

			assert_eq!(
				Some(&_Physics {
					target: Some(SkillTarget::Cursor(Cursor::TerrainHover))
				}),
				app.world().entity(entity).get::<_Physics>(),
			);
		}
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
	fn do_not_deref_slots_if_they_would_not_change(state: InputState) {
		let mut app = setup(_Input::from(std::iter::once((HandSlot::Left, state))));
		let entity = app
			.world_mut()
			.spawn((Player, _Loadout::from([SlotKey::from(HandSlot::Left)])))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(false),
			app.world()
				.entity(entity)
				.get::<_Loadout>()
				.map(|l| l.slots_dereferenced),
		);
	}

	#[test_case(InputState::just_pressed(); "on just pressed")]
	#[test_case(InputState::pressed(); "on pressed")]
	fn change_slots_if_target_would_not_change(state: InputState) {
		let mut app = setup(_Input::from(std::iter::once((HandSlot::Left, state))));
		let entity = app
			.world_mut()
			.spawn((Player, _Loadout::from([SlotKey::from(HandSlot::Left)])))
			.id();

		app.update();
		let mut input = app.world_mut().resource_mut::<_Input>();
		input.0.insert(ActionKey::from(HandSlot::Right), state);
		app.update();

		assert_eq!(
			Some(&HashSet::from([
				SlotKey::from(HandSlot::Left),
				SlotKey::from(HandSlot::Right)
			])),
			app.world()
				.entity(entity)
				.get::<_Loadout>()
				.map(|l| &l.slots),
		);
	}

	#[test_case(InputState::just_pressed(); "on just pressed")]
	#[test_case(InputState::pressed(); "on pressed")]
	fn change_target_even_if_skills_do_not_change(state: InputState) {
		let mut app = setup(_Input::from(std::iter::once((HandSlot::Left, state))));
		let entity = app
			.world_mut()
			.spawn((Player, _Loadout::from([SlotKey::from(HandSlot::Left)])))
			.id();

		app.update();
		let mut input = app.world_mut().resource_mut::<_Input>();
		input.0.insert(ActionKey::from(TerrainTargeting), state);
		app.update();

		assert_eq!(
			Some(SkillTarget::Cursor(Cursor::TerrainHover)),
			app.world()
				.entity(entity)
				.get::<_Loadout>()
				.and_then(|l| l.target),
		);
	}

	#[test_case(InputState::just_pressed(); "on just pressed")]
	#[test_case(InputState::pressed(); "on pressed")]
	fn do_not_deref_target_if_would_not_change(state: InputState) {
		let mut app = setup(_Input::from(std::iter::once((HandSlot::Left, state))));
		let entity = app
			.world_mut()
			.spawn((Player, _Loadout::from([SlotKey::from(HandSlot::Left)])))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(false),
			app.world()
				.entity(entity)
				.get::<_Loadout>()
				.map(|l| l.target_dereferenced),
		);
	}

	#[test_case(InputState::just_pressed(); "on just pressed")]
	#[test_case(InputState::pressed(); "on pressed")]
	fn do_not_deref_physics_if_target_would_not_change(state: InputState) {
		let mut app = setup(_Input::from(std::iter::once((HandSlot::Left, state))));
		let entity = app
			.world_mut()
			.spawn((
				Player,
				_Physics {
					target: Some(SkillTarget::Cursor(Cursor::Direction)),
				},
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<_Physics>>(),
		);
	}
}
