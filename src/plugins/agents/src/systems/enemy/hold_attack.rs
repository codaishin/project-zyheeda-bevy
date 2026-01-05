use crate::components::enemy::{Enemy, attack_phase::EnemyAttackPhase};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::GetContextMut,
	handles_loadout::{HeldSkillsMut, skills::Skills},
};

impl Enemy {
	pub(crate) fn hold_attack<TLoadout>(
		mut skills: StaticSystemParam<TLoadout>,
		attacking: Query<(Entity, &EnemyAttackPhase), Changed<EnemyAttackPhase>>,
		mut stopped_attacking: RemovedComponents<EnemyAttackPhase>,
	) where
		TLoadout: for<'c> GetContextMut<Skills, TContext<'c>: HeldSkillsMut>,
	{
		for (entity, phase) in &attacking {
			let Some(mut ctx) = TLoadout::get_context_mut(&mut skills, Skills { entity }) else {
				continue;
			};
			let held_skills = ctx.held_skills_mut();

			held_skills.clear();
			let EnemyAttackPhase::HoldSkill { key, .. } = phase else {
				continue;
			};
			held_skills.insert(*key);
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
	use common::{tools::action_key::slot::SlotKey, traits::handles_loadout::HeldSkills};
	use std::{collections::HashSet, time::Duration};
	use testing::SingleThreadedApp;

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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Enemy::hold_attack::<Query<&mut _Loadout>>);

		app
	}

	#[test]
	fn insert_held_skill() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EnemyAttackPhase::HoldSkill {
					key: SlotKey(42),
					holding: Duration::default(),
				},
				_Loadout::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Loadout::from([SlotKey(42)])),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test]
	fn override_held_skills() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EnemyAttackPhase::HoldSkill {
					key: SlotKey(42),
					holding: Duration::default(),
				},
				_Loadout::from([SlotKey(11)]),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Loadout::from([SlotKey(42)])),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test]
	fn clear_held_skill_when_in_cooldown() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
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
			Some(&_Loadout::default()),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
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
			.0
			.clear();
		app.update();

		assert_eq!(
			Some(&_Loadout::default()),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}

	#[test]
	fn act_again_if_attack_phase_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
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
			.0
			.clear();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<EnemyAttackPhase>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&_Loadout::from([SlotKey(42)])),
			app.world().entity(entity).get::<_Loadout>(),
		);
	}
}
