use bevy::prelude::*;
use common::traits::{
	accessors::get::GetterRef,
	handles_behaviors::{AttackConfig, AttackCoolDown, ConstantMovementSpeed},
};

#[derive(Component, Debug, PartialEq)]
pub struct Enemy {
	pub(crate) speed: ConstantMovementSpeed,
	pub(crate) attack: AttackConfig,
	pub(crate) cool_down: AttackCoolDown,
}

impl GetterRef<AttackConfig> for Enemy {
	fn get(&self) -> &AttackConfig {
		&self.attack
	}
}

impl GetterRef<AttackCoolDown> for Enemy {
	fn get(&self) -> &AttackCoolDown {
		&self.cool_down
	}
}

impl GetterRef<ConstantMovementSpeed> for Enemy {
	fn get(&self) -> &ConstantMovementSpeed {
		&self.speed
	}
}
