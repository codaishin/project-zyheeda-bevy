use bevy::ecs::system::EntityCommands;

use crate::{components::SlotKey, errors::Error};

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct MarkerMeta {
	pub insert_fn: fn(&mut EntityCommands, slot: SlotKey) -> Result<(), Error>,
	pub remove_fn: fn(&mut EntityCommands, slot: SlotKey) -> Result<(), Error>,
}

fn noop(_: &mut EntityCommands, _: SlotKey) -> Result<(), Error> {
	Ok(())
}

impl Default for MarkerMeta {
	fn default() -> Self {
		Self {
			insert_fn: noop,
			remove_fn: noop,
		}
	}
}
