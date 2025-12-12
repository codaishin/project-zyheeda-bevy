pub(crate) mod attack_config;
pub(crate) mod attack_phase;
pub(crate) mod attacking;
pub(crate) mod chasing;
pub(crate) mod void_sphere;

use crate::components::{enemy::attack_config::EnemyAttackConfig, movement_config::MovementConfig};
use bevy::prelude::*;
use common::{
	components::{
		collider_relationship::InteractionTarget,
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
	tools::Units,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[require(
	InteractionTarget,
	PersistentEntity,
	Transform,
	Visibility,
	IsBlocker = [Blocker::Character],
	MovementConfig,
	EnemyAttackConfig,
)]
pub struct Enemy {
	pub(crate) aggro_range: Units,
	pub(crate) attack_range: Units,
	pub(crate) min_target_distance: Option<Units>,
}
