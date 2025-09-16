pub(crate) mod enemy_type;
pub(crate) mod void_sphere;

use crate::components::enemy::enemy_type::EnemyTypeInternal;
use bevy::prelude::*;
use bevy_rapier3d::prelude::{GravityScale, RigidBody};
use common::{
	components::{
		collider_relationship::InteractionTarget,
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
	tools::{
		action_key::slot::SlotKey,
		aggro_range::AggroRange,
		attack_range::AttackRange,
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
	traits::handles_enemies::{EnemySkillUsage, EnemyTarget, EnemyType},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[require(
	InteractionTarget,
	PersistentEntity,
	Transform,
	Visibility,
	RigidBody = RigidBody::Dynamic,
	GravityScale = GravityScale(0.),
	IsBlocker = [Blocker::Character],
)]
pub struct Enemy {
	pub(crate) speed: Speed,
	pub(crate) movement_animation: Option<MovementAnimation>,
	pub(crate) aggro_range: AggroRange,
	pub(crate) attack_range: AttackRange,
	pub(crate) target: EnemyTarget,
	pub(crate) collider_radius: ColliderRadius,
	pub(crate) enemy_type: EnemyTypeInternal,
}

impl From<EnemyType> for Enemy {
	fn from(enemy_type: EnemyType) -> Self {
		EnemyTypeInternal::new_enemy(enemy_type)
	}
}

impl From<&Enemy> for Speed {
	fn from(enemy: &Enemy) -> Self {
		enemy.speed
	}
}

impl<'a> From<&'a Enemy> for Option<&'a MovementAnimation> {
	fn from(enemy: &'a Enemy) -> Self {
		enemy.movement_animation.as_ref()
	}
}

impl From<&Enemy> for AggroRange {
	fn from(enemy: &Enemy) -> Self {
		enemy.aggro_range
	}
}

impl From<&Enemy> for AttackRange {
	fn from(enemy: &Enemy) -> Self {
		enemy.attack_range
	}
}

impl From<&Enemy> for EnemyTarget {
	fn from(enemy: &Enemy) -> Self {
		enemy.target
	}
}

impl From<&Enemy> for ColliderRadius {
	fn from(enemy: &Enemy) -> Self {
		enemy.collider_radius
	}
}

impl EnemySkillUsage for Enemy {
	fn hold_skill(&self) -> Duration {
		self.enemy_type.hold_skill()
	}

	fn cool_down(&self) -> Duration {
		self.enemy_type.cool_down()
	}

	fn skill_key(&self) -> SlotKey {
		self.enemy_type.skill_key()
	}
}
