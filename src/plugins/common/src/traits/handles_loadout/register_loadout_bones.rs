use crate::tools::{action_key::slot::SlotKey, bone_name::BoneName, mesh_name::MeshName};
use bevy::prelude::*;
use macros::EntityKey;
use std::{collections::HashMap, ops::DerefMut};

#[derive(EntityKey)]
pub struct NoBonesRegistered {
	pub entity: Entity,
}

pub trait RegisterLoadoutBones {
	fn register_loadout_bones(
		&mut self,
		forearms: HashMap<BoneName, SlotKey>,
		hands: HashMap<BoneName, SlotKey>,
		essences: HashMap<MeshName, SlotKey>,
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
		essences: HashMap<MeshName, SlotKey>,
	) {
		self.deref_mut()
			.register_loadout_bones(forearms, hands, essences);
	}
}
