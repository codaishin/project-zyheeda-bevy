use crate::{
	components::{Active, Queued, Skill, SlotKey},
	errors::Error,
};
use bevy::{ecs::system::EntityCommands, utils::default};

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct SkillModify {
	pub modify_dual_fn: fn(&mut Skill<Active>, &mut Skill<Queued>),
	pub modify_single_fn: fn(&mut Skill<Active>, &mut Skill<Queued>),
}

fn no_modify(_: &mut Skill<Active>, _: &mut Skill<Queued>) {}

impl Default for SkillModify {
	fn default() -> Self {
		Self {
			modify_dual_fn: no_modify,
			modify_single_fn: no_modify,
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
