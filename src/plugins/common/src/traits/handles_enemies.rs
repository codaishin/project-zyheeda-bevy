use crate::tools::{Units, UnitsPerSecond};
use bevy::prelude::*;
use std::time::Duration;

use super::accessors::get::GetterRef;

pub trait HandlesEnemies {
	type TEnemy: GetterRef<ConstantMovementSpeed>
		+ GetterRef<AttackConfig>
		+ GetterRef<CoolDown>
		+ Component;
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct ConstantMovementSpeed(pub UnitsPerSecond);

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct CoolDown(pub Duration);

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct AttackConfig {
	pub aggro_range: Units,
	pub range: Units,
	pub method: AttackMethod,
	pub target: AttackTarget,
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum AttackMethod {
	#[default]
	VoidBeam,
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum AttackTarget {
	#[default]
	Player,
	Entity(Entity),
}
