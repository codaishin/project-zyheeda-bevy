use crate::components::agent::Agent;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::{GetChangedContext, GetContext, GetContextMut},
	handles_animations::{
		ActiveAnimationsMut,
		AnimationKey,
		AnimationPriority,
		Animations,
		AnimationsUnprepared,
	},
	handles_loadout::{ActiveSkill, ActiveSkills, skills::Skills},
};

impl Agent {
	pub(crate) fn animate_skills<TLoadout, TAnimations>(
		loadout: StaticSystemParam<TLoadout>,
		mut animations: StaticSystemParam<TAnimations>,
		agents: Query<Entity, With<Self>>,
	) -> Result<(), Vec<AnimationsUnprepared>>
	where
		TLoadout: for<'c> GetContext<Skills, TContext<'c>: ActiveSkills>,
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>,
	{
		let errors = agents
			.iter()
			.filter_map(|entity| {
				let key = Skills { entity };
				let loadout = TLoadout::get_changed_context(&loadout, key)?;

				let key = Animations { entity };
				let mut animations = TAnimations::get_context_mut(&mut animations, key)?;

				let active_animations = match animations.active_animations_mut(SkillAnimations) {
					Ok(active_animations) => active_animations,
					Err(error) => return Some(error),
				};

				let mut skills_to_animate = loadout.active_skills().filter(should_animate);
				match skills_to_animate.next() {
					Some(first) => {
						active_animations.insert(AnimationKey::Skill(first.key));
						for remaining in skills_to_animate {
							active_animations.insert(AnimationKey::Skill(remaining.key));
						}
					}
					None => {
						active_animations.clear();
					}
				}

				None
			})
			.collect::<Vec<_>>();

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}
}

fn should_animate(ActiveSkill { animate, .. }: &ActiveSkill) -> bool {
	*animate
}

struct SkillAnimations;

impl From<SkillAnimations> for AnimationPriority {
	fn from(_: SkillAnimations) -> Self {
		Self::High
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		assets::agent_config::AgentConfig,
		systems::player::animate_movement::tests::_Animations,
	};

	use super::*;
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{
			handles_animations::{AnimationKey, AnimationPriority},
			handles_loadout::{ActiveSkill, ActiveSkills},
			handles_map_generation::AgentType,
		},
	};
	use std::{
		collections::{HashMap, HashSet},
		iter::Copied,
		slice::Iter,
	};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Loadout {
		active: Vec<ActiveSkill>,
	}

	impl ActiveSkills for _Loadout {
		type TIter<'a>
			= Copied<Iter<'a, ActiveSkill>>
		where
			Self: 'a;

		fn active_skills(&self) -> Self::TIter<'_> {
			self.active.iter().copied()
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), Vec<AnimationsUnprepared>>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			Agent::animate_skills::<Query<Ref<_Loadout>>, Query<&mut _Animations>>.pipe(
				|In(result), mut commands: Commands| {
					commands.insert_resource(_Result(result));
				},
			),
		);

		app
	}

	#[test]
	fn insert_active_skills() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Agent {
					agent_type: AgentType::Player,
					config_handle: new_handle::<AgentConfig>(),
				},
				_Loadout {
					active: vec![
						ActiveSkill {
							key: SlotKey(42),
							animate: true,
						},
						ActiveSkill {
							key: SlotKey(11),
							animate: true,
						},
					],
				},
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
				AnimationPriority::High,
				HashSet::from([
					AnimationKey::Skill(SlotKey(42)),
					AnimationKey::Skill(SlotKey(11)),
				])
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn ignore_active_skills_with_animate_set_to_false() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Agent {
					agent_type: AgentType::Player,
					config_handle: new_handle::<AgentConfig>(),
				},
				_Loadout {
					active: vec![
						ActiveSkill {
							key: SlotKey(42),
							animate: true,
						},
						ActiveSkill {
							key: SlotKey(11),
							animate: false,
						},
					],
				},
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
				AnimationPriority::High,
				HashSet::from([AnimationKey::Skill(SlotKey(42))])
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn clear_active_skill_animations_when_no_skills_active() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Agent {
					agent_type: AgentType::Player,
					config_handle: new_handle::<AgentConfig>(),
				},
				_Loadout { active: vec![] },
				_Animations::Prepared(HashMap::from([
					(AnimationPriority::Low, HashSet::from([AnimationKey::Idle])),
					(
						AnimationPriority::Medium,
						HashSet::from([AnimationKey::Run]),
					),
					(
						AnimationPriority::High,
						HashSet::from([
							AnimationKey::Skill(SlotKey(42)),
							AnimationKey::Skill(SlotKey(11)),
						]),
					),
				])),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([
				(AnimationPriority::Low, HashSet::from([AnimationKey::Idle])),
				(
					AnimationPriority::Medium,
					HashSet::from([AnimationKey::Run]),
				),
				(AnimationPriority::High, HashSet::from([]),),
			])),),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn ignore_when_agent_missing() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Loadout {
					active: vec![ActiveSkill {
						key: SlotKey(42),
						animate: true,
					}],
				},
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations::default()),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Agent {
					agent_type: AgentType::Player,
					config_handle: new_handle::<AgentConfig>(),
				},
				_Loadout {
					active: vec![ActiveSkill {
						key: SlotKey(42),
						animate: true,
					}],
				},
				_Animations::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Animations::default());
		app.update();

		assert_eq!(
			Some(&_Animations::default()),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn act_again_if_loadout_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Agent {
					agent_type: AgentType::Player,
					config_handle: new_handle::<AgentConfig>(),
				},
				_Loadout {
					active: vec![ActiveSkill {
						key: SlotKey(42),
						animate: true,
					}],
				},
				_Animations::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Animations::default())
			.get_mut::<_Loadout>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&_Animations::Prepared(HashMap::from([(
				AnimationPriority::High,
				HashSet::from([AnimationKey::Skill(SlotKey(42))])
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}

	#[test]
	fn return_errors() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Agent {
					agent_type: AgentType::Player,
					config_handle: new_handle::<AgentConfig>(),
				},
				_Loadout { active: vec![] },
			))
			.id();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Animations::Unprepared(entity));

		app.update();

		assert_eq!(
			&_Result(Err(vec![AnimationsUnprepared { entity }])),
			app.world().resource::<_Result>(),
		);
	}
}
