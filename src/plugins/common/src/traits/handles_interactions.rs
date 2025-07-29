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
		config: BeamConfig,
		blocked_by: HashSet<Blocker>,
	},
	Fragile {
		destroyed_by: HashSet<Blocker>,
	},
}

#[derive(Debug, PartialEq, Clone)]
pub struct BeamConfig {
	pub source: PersistentEntity,
	pub target: PersistentEntity,
	pub range: Units,
}
