pub mod functions;
pub mod meta;

#[cfg(test)]
pub mod test_tools {
	use super::meta::MarkerModifyFn;
	use crate::{components::SlotKey, errors::Error};
	use bevy::{ecs::system::Commands, prelude::Entity};

	pub fn system(
		func: MarkerModifyFn,
		agent: Entity,
		slot: SlotKey,
	) -> impl FnMut(Commands) -> Result<(), Error> {
		move |mut commands| func(&mut commands.entity(agent), slot)
	}
}

pub struct Slow;

pub struct Fast;

pub struct Idle;

pub struct Dual;

#[derive(PartialEq, Debug)]
pub struct Left;

#[derive(PartialEq, Debug)]
pub struct Right;

#[derive(PartialEq, Debug)]
pub struct HandGun;

#[derive(PartialEq, Debug)]
pub struct Sword;
