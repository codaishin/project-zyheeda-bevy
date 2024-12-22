use crate::traits::insert_attack::InsertAttack;
use bevy::prelude::*;
use common::{
	tools::Units,
	traits::handles_enemies::{Attacker, EnemyAttack, EnemyConfig, EnemyTarget, Target},
};
use std::{sync::Arc, time::Duration};

#[derive(Component)]
pub struct Enemy {
	pub(crate) aggro_range: Units,
	pub(crate) attack_range: Units,
	pub(crate) target: EnemyTarget,
	pub(crate) attack: Arc<dyn InsertAttack + Sync + Send + 'static>,
	pub(crate) cool_down: Duration,
}

impl EnemyConfig for Enemy {
	fn attack_range(&self) -> Units {
		self.attack_range
	}

	fn aggro_range(&self) -> Units {
		self.aggro_range
	}

	fn target(&self) -> EnemyTarget {
		self.target
	}
}

impl EnemyAttack for Enemy {
	fn insert_attack(&self, entity: &mut EntityCommands, attacker: Attacker, target: Target) {
		self.attack.insert_attack(entity, attacker, target);
	}

	fn cool_down(&self) -> Duration {
		self.cool_down
	}
}
