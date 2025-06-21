use crate::traits::insert_attack::InsertAttack;
use bevy::prelude::*;
use common::{
	blocker::{Blocker, Blockers},
	components::{collider_relationship::InteractionTarget, persistent_entity::PersistentEntity},
	tools::{
		Units,
		aggro_range::AggroRange,
		attack_range::AttackRange,
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
	traits::{
		accessors::get::{Getter, GetterRefOptional},
		clamp_zero_positive::ClampZeroPositive,
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
pub struct Enemy {
	pub(crate) speed: Speed,
	pub(crate) movement_animation: Option<MovementAnimation>,
	pub(crate) aggro_range: AggroRange,
	pub(crate) attack_range: AttackRange,
	pub(crate) target: EnemyTarget,
	pub(crate) attack: Arc<dyn InsertAttack + Sync + Send + 'static>,
	pub(crate) cool_down: Duration,
	pub(crate) collider_radius: ColliderRadius,
}

impl Default for Enemy {
	fn default() -> Self {
		Self {
			speed: Default::default(),
			movement_animation: Default::default(),
			aggro_range: Default::default(),
			attack_range: Default::default(),
			target: Default::default(),
			attack: Arc::new(NoAttack),
			cool_down: Default::default(),
			collider_radius: ColliderRadius(Units::new(1.)),
		}
	}
}

impl Getter<Speed> for Enemy {
	fn get(&self) -> Speed {
		self.speed
	}
}

impl GetterRefOptional<MovementAnimation> for Enemy {
	fn get(&self) -> Option<&MovementAnimation> {
		self.movement_animation.as_ref()
	}
}

impl Getter<AggroRange> for Enemy {
	fn get(&self) -> AggroRange {
		self.aggro_range
	}
}

impl Getter<AttackRange> for Enemy {
	fn get(&self) -> AttackRange {
		self.attack_range
	}
}

impl Getter<EnemyTarget> for Enemy {
	fn get(&self) -> EnemyTarget {
		self.target
	}
}

impl Getter<ColliderRadius> for Enemy {
	fn get(&self) -> ColliderRadius {
		self.collider_radius
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

struct NoAttack;

impl InsertAttack for NoAttack {
	fn insert_attack(&self, _: &mut EntityCommands, _: Attacker, _: Target) {}
}
