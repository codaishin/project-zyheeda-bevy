use super::accessors::get::{Getter, GetterRefOptional};
use crate::{
	components::persistent_entity::PersistentEntity,
	tools::{
		aggro_range::AggroRange,
		attack_range::AttackRange,
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
};
use bevy::prelude::*;
use std::time::Duration;

pub trait HandlesEnemies {
	type TEnemy: Component
		+ Getter<Speed>
		+ GetterRefOptional<MovementAnimation>
		+ Getter<EnemyTarget>
		+ Getter<AggroRange>
		+ Getter<AttackRange>
		+ Getter<ColliderRadius>
		+ EnemyAttack;
}

pub trait EnemyAttack {
	fn insert_attack(&self, entity: &mut EntityCommands, attacker: Attacker, target: Target);
	fn cool_down(&self) -> Duration;
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum EnemyTarget {
	#[default]
	Player,
	Entity(PersistentEntity),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Attacker(pub PersistentEntity);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Target(pub PersistentEntity);
