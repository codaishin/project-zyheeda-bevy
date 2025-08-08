use crate::{
	components::{is_blocker::Blocker, persistent_entity::PersistentEntity},
	tools::Units,
};
use bevy::prelude::*;
use std::collections::HashSet;

pub trait HandlesInteractions {
	type TSystems: SystemSet;
	type TInteraction: Component + From<InteractAble>;

	const SYSTEMS: Self::TSystems;
}

#[derive(Debug, PartialEq, Clone)]
pub enum InteractAble {
	Beam {
		emitter: BeamEmitter,
		blocked_by: HashSet<Blocker>,
	},
	Fragile {
		destroyed_by: HashSet<Blocker>,
	},
}

#[derive(Debug, Clone)]
pub struct BeamEmitter {
	pub mounted_on: PersistentEntity,
	pub range: Units,
	pub insert_beam_model: fn(&mut EntityCommands),
}

impl PartialEq for BeamEmitter {
	fn eq(&self, other: &Self) -> bool {
		self.mounted_on == other.mounted_on
			&& self.range == other.range
			&& std::ptr::fn_addr_eq(self.insert_beam_model, other.insert_beam_model)
	}
}
