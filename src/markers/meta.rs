use crate::{components::SlotKey, errors::Error};
use bevy::ecs::system::EntityCommands;

pub type MarkerModifyFn = fn(&mut EntityCommands, slot: SlotKey) -> Result<(), Error>;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct MarkerMeta {
	pub insert_fn: MarkerModifyFn,
	pub remove_fn: MarkerModifyFn,
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
