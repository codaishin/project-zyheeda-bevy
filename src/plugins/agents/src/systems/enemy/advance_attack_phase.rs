use crate::components::enemy::{
	Enemy,
	attack_config::EnemyAttackConfig,
	attack_phase::EnemyAttackPhase,
};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};
use std::time::Duration;

impl Enemy {
	pub(crate) fn advance_attack_phase(
		In(delta): In<Duration>,
		mut commands: ZyheedaCommands,
		mut phases: Query<(Entity, &mut EnemyAttackPhase, &EnemyAttackConfig)>,
	) {
		for (entity, mut phase, config) in &mut phases {
			match phase.new_phase(delta, config) {
				Some(new_phase) => *phase = new_phase,
				None => {
					commands.try_apply_on(&entity, |mut e| {
						e.try_remove::<EnemyAttackPhase>();
					});
				}
			}
		}
	}
}

impl EnemyAttackPhase {
	fn new_phase(&self, delta: Duration, config: &EnemyAttackConfig) -> Option<Self> {
		use EnemyAttackPhase::*;

		match self {
			HoldSkill { holding, .. } if delta > *holding + config.cooldown => None,
			HoldSkill { holding, .. } if delta > *holding => {
				Some(Cooldown(config.cooldown - (delta - *holding)))
			}
			HoldSkill { holding, key } => Some(HoldSkill {
				holding: *holding - delta,
				key: *key,
			}),
			Cooldown(cooldown) if delta > *cooldown => None,
			Cooldown(cooldown) => Some(Cooldown(*cooldown - delta)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::enemy::{
		attack_config::EnemyAttackConfig,
		attack_phase::EnemyAttackPhase,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::action_key::slot::SlotKey;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn reduce_holding() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EnemyAttackPhase::HoldSkill {
					holding: Duration::from_secs(2),
					key: SlotKey(111),
				},
				EnemyAttackConfig::default(),
			))
			.id();

		app.world_mut()
			.run_system_once_with(Enemy::advance_attack_phase, Duration::from_secs(1))?;

		assert_eq!(
			Some(&EnemyAttackPhase::HoldSkill {
				holding: Duration::from_secs(1),
				key: SlotKey(111),
			}),
			app.world().entity(entity).get::<EnemyAttackPhase>(),
		);
		Ok(())
	}

	#[test]
	fn reduce_cooldown() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EnemyAttackPhase::Cooldown(Duration::from_secs(2)),
				EnemyAttackConfig::default(),
			))
			.id();

		app.world_mut()
			.run_system_once_with(Enemy::advance_attack_phase, Duration::from_secs(1))?;

		assert_eq!(
			Some(&EnemyAttackPhase::Cooldown(Duration::from_secs(1))),
			app.world().entity(entity).get::<EnemyAttackPhase>(),
		);
		Ok(())
	}

	#[test]
	fn transition_from_holding_to_cooldown() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EnemyAttackPhase::HoldSkill {
					holding: Duration::from_secs(2),
					key: SlotKey(111),
				},
				EnemyAttackConfig {
					cooldown: Duration::from_secs(10),
					..default()
				},
			))
			.id();

		app.world_mut()
			.run_system_once_with(Enemy::advance_attack_phase, Duration::from_secs(3))?;

		assert_eq!(
			Some(&EnemyAttackPhase::Cooldown(Duration::from_secs(9))),
			app.world().entity(entity).get::<EnemyAttackPhase>(),
		);
		Ok(())
	}

	#[test]
	fn remove_attack_phase_when_delta_greater_than_hold_and_cooldown() -> Result<(), RunSystemError>
	{
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EnemyAttackPhase::HoldSkill {
					holding: Duration::from_secs(2),
					key: SlotKey(111),
				},
				EnemyAttackConfig {
					cooldown: Duration::from_secs(3),
					..default()
				},
			))
			.id();

		app.world_mut()
			.run_system_once_with(Enemy::advance_attack_phase, Duration::from_secs(7))?;

		assert_eq!(None, app.world().entity(entity).get::<EnemyAttackPhase>());
		Ok(())
	}

	#[test]
	fn remove_done_cooldown() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EnemyAttackPhase::Cooldown(Duration::from_secs(2)),
				EnemyAttackConfig::default(),
			))
			.id();

		app.world_mut()
			.run_system_once_with(Enemy::advance_attack_phase, Duration::from_secs(3))?;

		assert_eq!(None, app.world().entity(entity).get::<EnemyAttackPhase>());
		Ok(())
	}
}
