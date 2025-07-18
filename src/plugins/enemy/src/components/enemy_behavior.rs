use crate::traits::insert_attack::InsertAttack;
use bevy::prelude::*;
use common::{
	blocker::{Blocker, Blockers},
	components::{collider_relationship::InteractionTarget, persistent_entity::PersistentEntity},
	tools::{
		aggro_range::AggroRange,
		attack_range::AttackRange,
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
	traits::{
		accessors::get::{Getter, GetterRefOptional},
		handles_enemies::{Attacker, EnemyAttack, EnemyTarget, Target},
	},
};
use std::{sync::Arc, time::Duration};

#[derive(Component, Clone)]
#[require(
	InteractionTarget,
	PersistentEntity,
	Transform,
	Visibility,
	Blockers = [Blocker::Character],
)]
pub struct EnemyBehavior {
	pub(crate) speed: Speed,
	pub(crate) movement_animation: Option<MovementAnimation>,
	pub(crate) aggro_range: AggroRange,
	pub(crate) attack_range: AttackRange,
	pub(crate) target: EnemyTarget,
	pub(crate) attack: Arc<dyn InsertAttack + Sync + Send + 'static>,
	pub(crate) cool_down: Duration,
	pub(crate) collider_radius: ColliderRadius,
}

impl Getter<Speed> for EnemyBehavior {
	fn get(&self) -> Speed {
		self.speed
	}
}

impl GetterRefOptional<MovementAnimation> for EnemyBehavior {
	fn get(&self) -> Option<&MovementAnimation> {
		self.movement_animation.as_ref()
	}
}

impl Getter<AggroRange> for EnemyBehavior {
	fn get(&self) -> AggroRange {
		self.aggro_range
	}
}

impl Getter<AttackRange> for EnemyBehavior {
	fn get(&self) -> AttackRange {
		self.attack_range
	}
}

impl Getter<EnemyTarget> for EnemyBehavior {
	fn get(&self) -> EnemyTarget {
		self.target
	}
}

impl Getter<ColliderRadius> for EnemyBehavior {
	fn get(&self) -> ColliderRadius {
		self.collider_radius
	}
}

impl EnemyAttack for EnemyBehavior {
	fn insert_attack(&self, entity: &mut EntityCommands, attacker: Attacker, target: Target) {
		self.attack.insert_attack(entity, attacker, target);
	}

	fn cool_down(&self) -> Duration {
		self.cool_down
	}
}
