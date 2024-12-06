use bevy::prelude::*;
use common::traits::{
	accessors::get::GetterRef,
	handles_enemies::{AttackConfig, ConstantMovementSpeed, CoolDown},
};

#[derive(Component, Debug, PartialEq)]
pub struct Enemy {
	pub(crate) speed: ConstantMovementSpeed,
	pub(crate) attack: AttackConfig,
	pub(crate) cool_down: CoolDown,
}

impl GetterRef<AttackConfig> for Enemy {
	fn get(&self) -> &AttackConfig {
		&self.attack
	}
}

impl GetterRef<CoolDown> for Enemy {
	fn get(&self) -> &CoolDown {
		&self.cool_down
	}
}

impl GetterRef<ConstantMovementSpeed> for Enemy {
	fn get(&self) -> &ConstantMovementSpeed {
		&self.speed
	}
}
