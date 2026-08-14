use bevy::prelude::*;
use common::prelude::*;
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct EnemyAttackConfig {
	pub(crate) key: SlotKey,
	pub(crate) hold: Duration,
	pub(crate) cooldown: Duration,
}
