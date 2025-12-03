use crate::tools::{action_key::slot::SlotKey, bone_name::BoneName};
use bevy::prelude::*;
use std::{collections::HashMap, ops::DerefMut};

pub struct NoBonesRegistered {
	pub entity: Entity,
}

impl From<NoBonesRegistered> for Entity {
	fn from(NoBonesRegistered { entity }: NoBonesRegistered) -> Self {
		entity
	}
}

pub trait RegisterLoadoutBones {
	fn register_loadout_bones(
		&mut self,
		forearms: HashMap<BoneName, SlotKey>,
		hands: HashMap<BoneName, SlotKey>,
		essences: HashMap<BoneName, SlotKey>,
	);
}

impl<T> RegisterLoadoutBones for T
where
	T: DerefMut<Target: RegisterLoadoutBones>,
{
	fn register_loadout_bones(
		&mut self,
		forearms: HashMap<BoneName, SlotKey>,
		hands: HashMap<BoneName, SlotKey>,
		essences: HashMap<BoneName, SlotKey>,
	) {
		self.deref_mut()
			.register_loadout_bones(forearms, hands, essences);
	}
}
