use crate::components::enemy::{
	Enemy,
	attack_config::EnemyAttackConfig,
	attack_phase::EnemyAttackPhase,
	attacking::Attacking,
};
use bevy::prelude::*;
use common::{self, traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl Enemy {
	pub(crate) fn begin_attack(
		mut commands: ZyheedaCommands,
		enemies: Query<(Entity, &EnemyAttackConfig, &Attacking), Without<EnemyAttackPhase>>,
	) {
		for (entity, config, attacking) in &enemies {
			if !attacking.has_los {
				continue;
			}

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(EnemyAttackPhase::HoldSkill {
					key: config.key,
					holding: config.hold,
				});
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::enemy::{attack_phase::EnemyAttackPhase, attacking::Attacking};
	use common::tools::action_key::slot::SlotKey;
	use std::time::Duration;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Enemy::begin_attack);

		app
	}

	#[test]
	fn attack_player_when_in_attack_range() {
		let mut app = setup();
		let player = app.world_mut().spawn_empty().id();
		let enemy = app
			.world_mut()
			.spawn((
				EnemyAttackConfig {
					key: SlotKey(11),
					hold: Duration::from_millis(111),
					..default()
				},
				Attacking {
					has_los: true,
					player,
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&EnemyAttackPhase::HoldSkill {
				key: SlotKey(11),
				holding: Duration::from_millis(111),
			}),
			app.world().entity(enemy).get::<EnemyAttackPhase>(),
		);
	}

	#[test]
	fn do_not_attack_player_when_no_los() {
		let mut app = setup();
		let player = app.world_mut().spawn_empty().id();
		let enemy = app
			.world_mut()
			.spawn((
				EnemyAttackConfig {
					key: SlotKey(11),
					..default()
				},
				Attacking {
					has_los: false,
					player,
				},
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(enemy).get::<EnemyAttackPhase>());
	}

	#[test]
	fn do_not_attack_player_when_already_attacking() {
		let mut app = setup();
		let player = app.world_mut().spawn_empty().id();
		let enemy = app
			.world_mut()
			.spawn((
				EnemyAttackConfig {
					key: SlotKey(11),
					..default()
				},
				Attacking {
					has_los: true,
					player,
				},
				EnemyAttackPhase::HoldSkill {
					key: SlotKey(22),
					holding: Duration::from_millis(3),
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&EnemyAttackPhase::HoldSkill {
				key: SlotKey(22),
				holding: Duration::from_millis(3),
			}),
			app.world().entity(enemy).get::<EnemyAttackPhase>()
		);
	}
}
