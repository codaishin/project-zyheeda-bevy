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
		Units,
		UnitsPerSecond,
		action_key::slot::SlotKey,
		aggro_range::AggroRange,
		attack_range::AttackRange,
		collider_radius::ColliderRadius,
		movement_animation::MovementAnimation,
		speed::Speed,
	},
	traits::{
		accessors::get::GetProperty,
		animation::Animation,
		handles_enemies::{EnemySkillUsage, EnemyTarget, EnemyType},
	},
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

impl GetProperty<Speed> for Enemy {
	fn get_property(&self) -> UnitsPerSecond {
		self.speed.0
	}
}

impl GetProperty<Option<MovementAnimation>> for Enemy {
	fn get_property(&self) -> Option<&'_ Animation> {
		self.movement_animation
			.as_ref()
			.map(|MovementAnimation(animation)| animation)
	}
}

impl GetProperty<AggroRange> for Enemy {
	fn get_property(&self) -> Units {
		self.aggro_range.0
	}
}

impl GetProperty<AttackRange> for Enemy {
	fn get_property(&self) -> Units {
		self.attack_range.0
	}
}

impl GetProperty<EnemyTarget> for Enemy {
	fn get_property(&self) -> EnemyTarget {
		self.target
	}
}

impl GetProperty<ColliderRadius> for Enemy {
	fn get_property(&self) -> Units {
		self.collider_radius.0
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
