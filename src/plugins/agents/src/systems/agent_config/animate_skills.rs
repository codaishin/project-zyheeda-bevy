use crate::components::agent_config::AgentConfig;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::{GetChangedContext, GetContext, GetContextMut},
		handles_animations::{
			ActiveAnimationsMut,
			AnimationKey,
			AnimationPriority,
			Animations,
			SkillAnimation,
		},
		handles_loadout::{ActiveSkill, ActiveSkills, skills::Skills},
	},
};

impl AgentConfig {
	pub(crate) fn animate_skills<TLoadout, TAnimations>(
		loadout: StaticSystemParam<TLoadout>,
		mut animations: StaticSystemParam<TAnimations>,
		agents: Query<Entity, With<Self>>,
	) where
		TLoadout: for<'c> GetContext<Skills, TContext<'c>: ActiveSkills>,
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: ActiveAnimationsMut>,
	{
		for entity in agents {
			let key = Skills { entity };
			let Some(loadout) = TLoadout::get_changed_context(&loadout, key) else {
				continue;
			};

			let key = Animations { entity };
			let Some(mut animations) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			let active_animations = animations.active_animations_mut(SkillAnimations);

			let mut skills_to_animate = loadout.active_skills().filter_map(with_animation);
			match skills_to_animate.next() {
				Some((slot, animation)) => {
					active_animations.insert(AnimationKey::Skill { slot, animation });
					for (slot, animation) in skills_to_animate {
						active_animations.insert(AnimationKey::Skill { slot, animation });
					}
				}
				None => {
					active_animations.clear();
				}
			}
		}
	}
}

fn with_animation(
	ActiveSkill { key, animation }: ActiveSkill,
) -> Option<(SlotKey, SkillAnimation)> {
	Some((key, animation?))
}

struct SkillAnimations;

impl From<SkillAnimations> for AnimationPriority {
	fn from(_: SkillAnimations) -> Self {
		Self::High
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		assets::agent_config::AgentConfigAsset,
		systems::player::animate_movement::tests::_Animations,
	};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{
			handles_animations::{AnimationKey, AnimationPriority},
			handles_loadout::{ActiveSkill, ActiveSkills},
		},
	};
	use std::{collections::HashMap, iter::Copied, slice::Iter};
	use testing::{SingleThreadedApp, new_handle};
	use zyheeda_core::prelude::*;

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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			AgentConfig::animate_skills::<Query<Ref<_Loadout>>, Query<&mut _Animations>>,
		);

		app
	}

	#[test]
	fn insert_active_skills() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				AgentConfig {
					config_handle: new_handle::<AgentConfigAsset>(),
				},
				_Loadout {
					active: vec![
						ActiveSkill {
							key: SlotKey(42),
							animation: Some(SkillAnimation::Shoot),
						},
						ActiveSkill {
							key: SlotKey(11),
							// FIXME: use different animation once `SkillAnimations` has more variants
							animation: Some(SkillAnimation::Shoot),
						},
					],
				},
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations(HashMap::from([(
				AnimationPriority::High,
				OrderedSet::from([
					AnimationKey::Skill {
						slot: SlotKey(42),
						animation: SkillAnimation::Shoot
					},
					AnimationKey::Skill {
						slot: SlotKey(11),
						animation: SkillAnimation::Shoot
					},
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
				AgentConfig {
					config_handle: new_handle::<AgentConfigAsset>(),
				},
				_Loadout {
					active: vec![
						ActiveSkill {
							key: SlotKey(42),
							animation: Some(SkillAnimation::Shoot),
						},
						ActiveSkill {
							key: SlotKey(11),
							animation: None,
						},
					],
				},
				_Animations::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations(HashMap::from([(
				AnimationPriority::High,
				OrderedSet::from([AnimationKey::Skill {
					slot: SlotKey(42),
					animation: SkillAnimation::Shoot
				}])
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
				AgentConfig {
					config_handle: new_handle::<AgentConfigAsset>(),
				},
				_Loadout { active: vec![] },
				_Animations(HashMap::from([
					(
						AnimationPriority::Low,
						OrderedSet::from([AnimationKey::Idle]),
					),
					(
						AnimationPriority::Medium,
						OrderedSet::from([AnimationKey::Run]),
					),
					(
						AnimationPriority::High,
						OrderedSet::from([
							AnimationKey::Skill {
								slot: SlotKey(42),
								animation: SkillAnimation::Shoot,
							},
							AnimationKey::Skill {
								slot: SlotKey(11),
								animation: SkillAnimation::Shoot,
							},
						]),
					),
				])),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Animations(HashMap::from([
				(
					AnimationPriority::Low,
					OrderedSet::from([AnimationKey::Idle])
				),
				(
					AnimationPriority::Medium,
					OrderedSet::from([AnimationKey::Run]),
				),
				(AnimationPriority::High, OrderedSet::from([]),),
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
						animation: Some(SkillAnimation::Shoot),
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
				AgentConfig {
					config_handle: new_handle::<AgentConfigAsset>(),
				},
				_Loadout {
					active: vec![ActiveSkill {
						key: SlotKey(42),
						animation: Some(SkillAnimation::Shoot),
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
				AgentConfig {
					config_handle: new_handle::<AgentConfigAsset>(),
				},
				_Loadout {
					active: vec![ActiveSkill {
						key: SlotKey(42),
						animation: Some(SkillAnimation::Shoot),
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
			Some(&_Animations(HashMap::from([(
				AnimationPriority::High,
				OrderedSet::from([AnimationKey::Skill {
					slot: SlotKey(42),
					animation: SkillAnimation::Shoot
				}])
			)]))),
			app.world().entity(entity).get::<_Animations>(),
		);
	}
}
