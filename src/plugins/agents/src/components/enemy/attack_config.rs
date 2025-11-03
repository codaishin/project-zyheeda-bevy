use bevy::prelude::*;
use common::tools::action_key::slot::SlotKey;
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct EnemyAttackConfig {
	pub(crate) key: SlotKey,
	pub(crate) hold: Duration,
	pub(crate) cooldown: Duration,
}
