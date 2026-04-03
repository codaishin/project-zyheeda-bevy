use crate::components::enemy::{Enemy, attack_phase::EnemyAttackPhase, attacking::Attacking};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::GetContextMut,
	handles_loadout::{CurrentTargetMut, HeldSkillsMut, skills::Skills},
	handles_skill_physics::SkillTarget,
};

impl Enemy {
	pub(crate) fn hold_attack<TLoadout>(
		mut skills: StaticSystemParam<TLoadout>,
		attacking: Query<(Entity, &EnemyAttackPhase, &Attacking), Changed<EnemyAttackPhase>>,
		mut stopped_attacking: RemovedComponents<EnemyAttackPhase>,
	) where
		TLoadout: for<'c> GetContextMut<Skills, TContext<'c>: HeldSkillsMut>,
	{
		for (entity, phase, Attacking { player, .. }) in &attacking {
			let Some(mut ctx) = TLoadout::get_context_mut(&mut skills, Skills { entity }) else {
				continue;
			};
			ctx.held_skills_mut().clear();

			let EnemyAttackPhase::HoldSkill { key, .. } = phase else {
				continue;
			};

			ctx.held_skills_mut().insert(*key);
			*ctx.current_target_mut() = Some(SkillTarget::Entity(*player));
		}

		for entity in stopped_attacking.read() {
			let Some(mut ctx) = TLoadout::get_context_mut(&mut skills, Skills { entity }) else {
				continue;
			};

			ctx.held_skills_mut().clear();
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::enemy::attack_phase::EnemyAttackPhase;
	use common::{
		components::persistent_entity::PersistentEntity,
		tools::action_key::slot::SlotKey,
		traits::{
			handles_loadout::{CurrentTarget, CurrentTargetMut, HeldSkills},
			handles_skill_physics::SkillTarget,
		},
	};
	use std::{collections::HashSet, sync::LazyLock, time::Duration};
	use testing::SingleThreadedApp;

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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Enemy::hold_attack::<Query<&mut _Loadout>>);

		app
	}

	static PLAYER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[test]
	fn insert_held_skill() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Attacking {
					player: *PLAYER,
					has_los: true,
				},
				EnemyAttackPhase::HoldSkill {
					key: SlotKey(42),
					holding: Duration::default(),
				},
				_Loadout::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Loadout::from([SlotKey(42)]).with_target(Some(SkillTarget::Entity(*PLAYER)))),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test]
	fn override_held_skills() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Attacking {
					player: *PLAYER,
					has_los: true,
				},
				EnemyAttackPhase::HoldSkill {
					key: SlotKey(42),
					holding: Duration::default(),
				},
				_Loadout::from([SlotKey(11)]),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Loadout::from([SlotKey(42)]).with_target(Some(SkillTarget::Entity(*PLAYER)))),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test]
	fn clear_held_skill_when_in_cooldown() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Attacking {
					player: *PLAYER,
					has_los: true,
				},
				EnemyAttackPhase::Cooldown(Duration::ZERO),
				_Loadout::from([SlotKey(11)]),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Loadout::default()),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test]
	fn clear_held_skills_when_removing_attack_phase() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Attacking {
					player: *PLAYER,
					has_los: true,
				},
				EnemyAttackPhase::HoldSkill {
					key: SlotKey(42),
					holding: Duration::default(),
				},
				_Loadout::from([SlotKey(11)]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<EnemyAttackPhase>();
		app.update();

		assert_eq!(
			Some(&_Loadout::default().with_target(Some(SkillTarget::Entity(*PLAYER)))),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Attacking {
					player: *PLAYER,
					has_los: true,
				},
				EnemyAttackPhase::HoldSkill {
					key: SlotKey(42),
					holding: Duration::default(),
				},
				_Loadout::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Loadout>()
			.unwrap()
			.slots
			.clear();
		app.update();

		assert_eq!(
			Some(&_Loadout::default().with_target(Some(SkillTarget::Entity(*PLAYER)))),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test]
	fn act_again_if_attack_phase_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Attacking {
					player: *PLAYER,
					has_los: true,
				},
				EnemyAttackPhase::HoldSkill {
					key: SlotKey(42),
					holding: Duration::default(),
				},
				_Loadout::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Loadout>()
			.unwrap()
			.slots
			.clear();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<EnemyAttackPhase>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&_Loadout::from([SlotKey(42)]).with_target(Some(SkillTarget::Entity(*PLAYER)))),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}
}
