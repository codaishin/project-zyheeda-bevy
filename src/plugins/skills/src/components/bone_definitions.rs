use bevy::prelude::*;
use common::{
	tools::{action_key::slot::SlotKey, bone_name::BoneName},
	traits::{
		bone_key::BoneKey,
		loadout::{ItemName, LoadoutConfig},
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct BoneDefinitions {
	pub(crate) forearms: HashMap<BoneName, SlotKey>,
	pub(crate) hands: HashMap<BoneName, SlotKey>,
	pub(crate) essences: HashMap<BoneName, SlotKey>,
	// FIXME: Once skills do not rely on agents any more, remove the below fields,
	//        because default loadout is inserted directly through `SkillsPlugin::TLoadoutPrep`
	pub(crate) default_inventory_loadout: Vec<Option<ItemName>>,
	pub(crate) default_slots_loadout: Vec<(SlotKey, Option<ItemName>)>,
}

impl BoneKey<ForearmSlot> for BoneDefinitions {
	fn bone_key(&self, value: &str) -> Option<ForearmSlot> {
		self.forearms.get(value).copied().map(ForearmSlot)
	}
}

impl BoneKey<HandSlot> for BoneDefinitions {
	fn bone_key(&self, value: &str) -> Option<HandSlot> {
		self.hands.get(value).copied().map(HandSlot)
	}
}

impl BoneKey<EssenceSlot> for BoneDefinitions {
	fn bone_key(&self, value: &str) -> Option<EssenceSlot> {
		self.essences.get(value).copied().map(EssenceSlot)
	}
}

impl LoadoutConfig for BoneDefinitions {
	fn inventory(&self) -> impl Iterator<Item = Option<ItemName>> {
		self.default_inventory_loadout.iter().cloned()
	}

	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<ItemName>)> {
		self.default_slots_loadout.iter().cloned()
	}
}
