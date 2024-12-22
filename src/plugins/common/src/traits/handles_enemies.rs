use crate::tools::Units;
use bevy::prelude::*;
use std::time::Duration;

pub trait HandlesEnemies {
	type TEnemy: Component + EnemyConfig + EnemyAttack;
}

pub trait EnemyConfig {
	fn attack_range(&self) -> Units;
	fn aggro_range(&self) -> Units;
	fn target(&self) -> EnemyTarget;
}

pub trait EnemyAttack {
	fn insert_attack(&self, entity: &mut EntityCommands, attacker: Attacker, target: Target);
	fn cool_down(&self) -> Duration;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EnemyTarget {
	Player,
	Enemy(Entity),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Attacker(pub Entity);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Target(pub Entity);
