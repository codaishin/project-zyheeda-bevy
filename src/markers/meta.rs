use crate::{
	components::{Active, Queued, Skill, SlotKey},
	errors::Error,
};
use bevy::{ecs::system::EntityCommands, utils::default};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tag {
	HandGun,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Chain {
	pub can_chain: fn(&Skill<Active>, &Skill<Queued>) -> bool,
	pub modify_dual: fn(&mut Skill<Active>, &mut Skill<Queued>),
	pub modify_single: fn(&mut Skill<Active>, &mut Skill<Queued>),
}

fn no_chain(_: &Skill<Active>, _: &Skill<Queued>) -> bool {
	false
}

fn no_modify(_: &mut Skill<Active>, _: &mut Skill<Queued>) {}

impl Default for Chain {
	fn default() -> Self {
		Self {
			can_chain: no_chain,
			modify_dual: no_modify,
			modify_single: no_modify,
		}
	}
}

pub type MarkerModifyFn = fn(&mut EntityCommands, slot: SlotKey) -> Result<(), Error>;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct MarkerMeta {
	pub tag: Option<Tag>,
	pub insert_fn: MarkerModifyFn,
	pub remove_fn: MarkerModifyFn,
	pub chain: Chain,
}

fn noop(_: &mut EntityCommands, _: SlotKey) -> Result<(), Error> {
	Ok(())
}

impl Default for MarkerMeta {
	fn default() -> Self {
		Self {
			tag: None,
			insert_fn: noop,
			remove_fn: noop,
			chain: default(),
		}
	}
}
