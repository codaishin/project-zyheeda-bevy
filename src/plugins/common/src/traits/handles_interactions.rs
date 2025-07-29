use crate::{components::is_blocker::Blocker, tools::Units};
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
		range: Units,
		blocked_by: HashSet<Blocker>,
	},
	Fragile {
		destroyed_by: HashSet<Blocker>,
	},
}
