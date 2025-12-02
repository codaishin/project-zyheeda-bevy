use crate::tools::{action_key::slot::SlotKey, bone_name::BoneName};
use bevy::prelude::*;
use std::collections::HashMap;

pub trait RegisterLoadoutBones {
	fn register_loadout_bones(
		&mut self,
		forearms: HashMap<BoneName, SlotKey>,
		hands: HashMap<BoneName, SlotKey>,
		essences: HashMap<BoneName, SlotKey>,
	);
}

pub struct NoBonesRegistered {
	pub entity: Entity,
}
