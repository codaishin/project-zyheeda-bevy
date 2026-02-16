use bevy::prelude::*;
use common::tools::action_key::slot::SlotKey;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[savable_component(id = "enemy attack phase")]
pub(crate) enum EnemyAttackPhase {
	HoldSkill { key: SlotKey, holding: Duration },
	Cooldown(Duration),
}
