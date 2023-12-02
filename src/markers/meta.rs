use bevy::ecs::system::EntityCommands;

use crate::{
	components::{Active, Queued, Skill, SlotKey},
	errors::Error,
};

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct MarkerMeta {
	pub insert_fn: fn(&mut EntityCommands, slot: SlotKey) -> Result<(), Error>,
	pub remove_fn: fn(&mut EntityCommands, slot: SlotKey) -> Result<(), Error>,
	pub soft_override: fn(&mut Skill<Active>, &mut Skill<Queued>) -> bool,
}

fn noop(_: &mut EntityCommands, _: SlotKey) -> Result<(), Error> {
	Ok(())
}

fn no_soft_override(_running: &mut Skill<Active>, _new: &mut Skill<Queued>) -> bool {
	false
}

impl Default for MarkerMeta {
	fn default() -> Self {
		Self {
			insert_fn: noop,
			remove_fn: noop,
			soft_override: no_soft_override,
		}
	}
}
