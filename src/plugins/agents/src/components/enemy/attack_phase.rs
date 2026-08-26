use bevy::prelude::*;
use common::prelude::*;
use macros::{SavableComponent, serde_model};
use std::time::Duration;

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[savable_component(id = "enemy attack phase")]
pub(crate) enum EnemyAttackPhase {
	HoldSkill { key: SlotKey, holding: Duration },
	Cooldown(Duration),
}
