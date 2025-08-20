use crate::components::{Attack, attacking::Attacking, skill_usage::SkillUsage};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, handles_enemies::EnemySkillUsage},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::HashSet;

macro_rules! empty {
	(started($skill:expr)) => {
		$skill.started_holding.is_empty()
	};
	(all($skill:expr)) => {
		$skill.started_holding.is_empty() && $skill.holding.is_empty()
	};
}

impl SkillUsage {
	pub(crate) fn enemy<TEnemy>(
		mut commands: ZyheedaCommands,
		mut new_attacks: Query<(Entity, &TEnemy, &mut SkillUsage, &Attack), Without<Attacking>>,
		mut ongoing_attacks: Query<(&mut SkillUsage, &Attacking), With<TEnemy>>,
		mut removed_attacks: RemovedComponents<Attack>,
		enemies: Query<(), With<TEnemy>>,
	) where
		TEnemy: Component + EnemySkillUsage,
	{
		for (entity, enemy, mut skill, Attack(_)) in &mut new_attacks {
			skill.started_holding = HashSet::from([enemy.skill_key()]);
			skill.holding = HashSet::from([enemy.skill_key()]);

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(Attacking::Hold {
					remaining: enemy.hold_skill(),
					cool_down: enemy.cool_down(),
				});
			});
		}

		for (mut skill, attacking) in &mut ongoing_attacks {
			match attacking {
				Attacking::Hold { .. } if !empty!(started(skill)) => {
					skill.started_holding.clear();
				}
				Attacking::CoolDown { .. } if !empty!(all(skill)) => {
					skill.started_holding.clear();
					skill.holding.clear();
				}
				_ => {}
			};
		}

		for entity in removed_attacks.read().filter(|e| enemies.contains(*e)) {
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(SkillUsage::default());
				e.try_remove::<Attacking>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		components::persistent_entity::PersistentEntity,
		tools::action_key::slot::SlotKey,
		traits::handles_enemies::EnemySkillUsage,
	};
	use std::{collections::HashSet, time::Duration};
	use testing::{IsChanged, SingleThreadedApp};

	#[derive(Component, Default)]
	#[require(SkillUsage)]
	struct _Enemy {
		hold_skill: Duration,
		cool_down: Duration,
		skill_key: SlotKey,
	}

	impl EnemySkillUsage for _Enemy {
		fn hold_skill(&self) -> Duration {
			self.hold_skill
		}

		fn cool_down(&self) -> Duration {
			self.cool_down
		}

		fn skill_key(&self) -> SlotKey {
			self.skill_key
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(SkillUsage::enemy::<_Enemy>, IsChanged::<SkillUsage>::detect).chain(),
		);

		app
	}

	mod new_attack {
		use super::*;

		#[test]
		fn set_hold_skill() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Enemy {
						skill_key: SlotKey(11),
						..default()
					},
					SkillUsage {
						started_holding: HashSet::from([SlotKey(44), SlotKey(55), SlotKey(66)]),
						holding: HashSet::from([SlotKey(44), SlotKey(55), SlotKey(66)]),
					},
					Attack(PersistentEntity::default()),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&SkillUsage {
					holding: HashSet::from([SlotKey(11)]),
					started_holding: HashSet::from([SlotKey(11)])
				}),
				app.world().entity(entity).get::<SkillUsage>()
			);
		}

		#[test]
		fn insert_attacking_components() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Enemy {
						hold_skill: Duration::from_millis(42),
						cool_down: Duration::from_millis(11),
						..default()
					},
					Attack(PersistentEntity::default()),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Attacking::Hold {
					remaining: Duration::from_millis(42),
					cool_down: Duration::from_millis(11),
				}),
				app.world().entity(entity).get::<Attacking>(),
			);
		}
	}

	mod attack_hold {
		use super::*;

		#[test]
		fn clear_started_holding() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Enemy {
						skill_key: SlotKey(11),
						..default()
					},
					SkillUsage {
						started_holding: HashSet::from([SlotKey(11), SlotKey(12), SlotKey(13)]),
						holding: HashSet::from([SlotKey(11)]),
					},
					Attack(PersistentEntity::default()),
					Attacking::Hold {
						remaining: Duration::default(),
						cool_down: Duration::default(),
					},
				))
				.id();

			app.update();

			assert_eq!(
				Some(&SkillUsage {
					started_holding: HashSet::from([]),
					holding: HashSet::from([SlotKey(11)]),
				}),
				app.world().entity(entity).get::<SkillUsage>()
			);
		}

		#[test]
		fn do_not_clear_when_started_holding_empty() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Enemy {
						skill_key: SlotKey(11),
						..default()
					},
					SkillUsage {
						started_holding: HashSet::from([]),
						holding: HashSet::from([SlotKey(11)]),
					},
				))
				.id();

			app.update();
			app.world_mut().entity_mut(entity).insert((
				Attack(PersistentEntity::default()),
				Attacking::Hold {
					remaining: Duration::default(),
					cool_down: Duration::default(),
				},
			));
			app.update();

			assert_eq!(
				Some(&IsChanged::FALSE),
				app.world().entity(entity).get::<IsChanged<SkillUsage>>()
			);
		}
	}

	mod attack_cool_down {
		use super::*;

		#[test]
		fn clear_skill_usage() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Enemy {
						skill_key: SlotKey(11),
						..default()
					},
					SkillUsage {
						started_holding: HashSet::from([SlotKey(11), SlotKey(12), SlotKey(13)]),
						holding: HashSet::from([SlotKey(11), SlotKey(12), SlotKey(13)]),
					},
					Attack(PersistentEntity::default()),
					Attacking::CoolDown {
						remaining: Duration::default(),
					},
				))
				.id();

			app.update();

			assert_eq!(
				Some(&SkillUsage::default()),
				app.world().entity(entity).get::<SkillUsage>()
			);
		}

		#[test]
		fn do_not_clear_when_empty() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Enemy {
						skill_key: SlotKey(11),
						..default()
					},
					SkillUsage {
						started_holding: HashSet::from([]),
						holding: HashSet::from([]),
					},
				))
				.id();

			app.update();
			app.world_mut().entity_mut(entity).insert((
				Attack(PersistentEntity::default()),
				Attacking::CoolDown {
					remaining: Duration::default(),
				},
			));
			app.update();

			assert_eq!(
				Some(&IsChanged::FALSE),
				app.world().entity(entity).get::<IsChanged<SkillUsage>>()
			);
		}

		#[test]
		fn do_clear_when_started_holding_not_empty() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Enemy {
						skill_key: SlotKey(11),
						..default()
					},
					SkillUsage {
						started_holding: HashSet::from([SlotKey(42)]),
						holding: HashSet::from([]),
					},
				))
				.id();

			app.update();
			app.world_mut().entity_mut(entity).insert((
				Attack(PersistentEntity::default()),
				Attacking::CoolDown {
					remaining: Duration::default(),
				},
			));
			app.update();

			assert_eq!(
				Some(&SkillUsage::default()),
				app.world().entity(entity).get::<SkillUsage>()
			);
		}

		#[test]
		fn do_clear_when_holding_not_empty() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Enemy {
						skill_key: SlotKey(11),
						..default()
					},
					SkillUsage {
						started_holding: HashSet::from([]),
						holding: HashSet::from([SlotKey(42)]),
					},
				))
				.id();

			app.update();
			app.world_mut().entity_mut(entity).insert((
				Attack(PersistentEntity::default()),
				Attacking::CoolDown {
					remaining: Duration::default(),
				},
			));
			app.update();

			assert_eq!(
				Some(&SkillUsage::default()),
				app.world().entity(entity).get::<SkillUsage>()
			);
		}
	}

	mod stop_attack {
		use super::*;

		#[test]
		fn remove_attacking_components() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Enemy {
						hold_skill: Duration::from_millis(42),
						cool_down: Duration::from_millis(11),
						..default()
					},
					Attack(PersistentEntity::default()),
					Attacking::Hold {
						remaining: Duration::from_millis(42),
						cool_down: Duration::from_millis(11),
					},
				))
				.id();

			app.update();
			app.world_mut().entity_mut(entity).remove::<Attack>();
			app.update();

			assert_eq!(None, app.world().entity(entity).get::<Attacking>());
		}

		#[test]
		fn do_not_remove_attacking_components_when_enemy_missing() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Attack(PersistentEntity::default()),
					Attacking::Hold {
						remaining: Duration::from_millis(42),
						cool_down: Duration::from_millis(11),
					},
				))
				.id();

			app.update();
			app.world_mut().entity_mut(entity).remove::<Attack>();
			app.update();

			assert_eq!(
				Some(&Attacking::Hold {
					remaining: Duration::from_millis(42),
					cool_down: Duration::from_millis(11),
				}),
				app.world().entity(entity).get::<Attacking>()
			);
		}

		#[test]
		fn reset_skill_usage() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Enemy {
						hold_skill: Duration::from_millis(42),
						cool_down: Duration::from_millis(11),
						..default()
					},
					SkillUsage {
						started_holding: HashSet::from([SlotKey(244)]),
						holding: HashSet::from([SlotKey(233)]),
					},
					Attack(PersistentEntity::default()),
					Attacking::Hold {
						remaining: Duration::from_millis(42),
						cool_down: Duration::from_millis(11),
					},
				))
				.id();

			app.update();
			app.world_mut().entity_mut(entity).remove::<Attack>();
			app.update();

			assert_eq!(
				Some(&SkillUsage::default()),
				app.world().entity(entity).get::<SkillUsage>()
			);
		}

		#[test]
		fn do_not_reset_skill_usage_when_enemy_missing() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					SkillUsage {
						started_holding: HashSet::from([SlotKey(244)]),
						holding: HashSet::from([SlotKey(233)]),
					},
					Attack(PersistentEntity::default()),
					Attacking::Hold {
						remaining: Duration::from_millis(42),
						cool_down: Duration::from_millis(11),
					},
				))
				.id();

			app.update();
			app.world_mut().entity_mut(entity).remove::<Attack>();
			app.update();

			assert_eq!(
				Some(&SkillUsage {
					started_holding: HashSet::from([SlotKey(244)]),
					holding: HashSet::from([SlotKey(233)]),
				}),
				app.world().entity(entity).get::<SkillUsage>()
			);
		}
	}
}
