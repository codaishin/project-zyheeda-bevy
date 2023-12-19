use crate::{
	components::{Active, Queued, Skill, SlotKey},
	errors::Error,
};
use bevy::{ecs::system::EntityCommands, utils::default};

pub type UpdateFn = fn(&Skill<Active>, &Skill<Queued>) -> (Skill<Active>, Skill<Queued>);

fn no_update(running: &Skill<Active>, new: &Skill<Queued>) -> (Skill<Active>, Skill<Queued>) {
	(*running, *new)
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct SkillModify {
	pub update_dual_fn: UpdateFn,
	pub update_single_fn: UpdateFn,
}

impl Default for SkillModify {
	fn default() -> Self {
		Self {
			update_dual_fn: no_update,
			update_single_fn: no_update,
		}
	}
}

pub type MarkerModifyFn = fn(&mut EntityCommands, slot: SlotKey) -> Result<(), Error>;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct MarkerMeta {
	pub insert_fn: MarkerModifyFn,
	pub remove_fn: MarkerModifyFn,
	pub skill_modify: SkillModify,
}

fn noop(_: &mut EntityCommands, _: SlotKey) -> Result<(), Error> {
	Ok(())
}

impl Default for MarkerMeta {
	fn default() -> Self {
		Self {
			insert_fn: noop,
			remove_fn: noop,
			skill_modify: default(),
		}
	}
}
