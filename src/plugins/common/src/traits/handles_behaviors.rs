use super::accessors::get::GetterRef;
use crate::tools::{Units, UnitsPerSecond};
use bevy::prelude::*;
use std::time::Duration;

pub trait HandlesBehaviors {
	/// [!WARNING]
	/// Using this for multiple agents may result in undefined behavior
	fn register_camera_orbit_for<TAgent>(app: &mut App)
	where
		TAgent: Component;

	fn register_enemies_for<TPlayer, TEnemy>(app: &mut App)
	where
		TPlayer: Component,
		TEnemy: Component
			+ GetterRef<AttackConfig>
			+ GetterRef<AttackCoolDown>
			+ GetterRef<ConstantMovementSpeed>;
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct ConstantMovementSpeed(pub UnitsPerSecond);

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct AttackCoolDown(pub Duration);

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
