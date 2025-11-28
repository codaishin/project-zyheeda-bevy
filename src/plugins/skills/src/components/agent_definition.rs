use bevy::prelude::*;
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		bone_key::BoneKey,
		loadout::{ItemName, LoadoutConfig},
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct AgentDefinition {
	pub(crate) forearms: HashMap<String, ForearmSlot>,
	pub(crate) hands: HashMap<String, HandSlot>,
	pub(crate) essences: HashMap<String, EssenceSlot>,
	pub(crate) default_inventory_loadout: Vec<Option<ItemName>>,
	pub(crate) default_slots_loadout: Vec<(SlotKey, Option<ItemName>)>,
}

impl BoneKey<ForearmSlot> for AgentDefinition {
	fn bone_key(&self, value: &str) -> Option<ForearmSlot> {
		self.forearms.get(value).copied()
	}
}

impl BoneKey<HandSlot> for AgentDefinition {
	fn bone_key(&self, value: &str) -> Option<HandSlot> {
		self.hands.get(value).copied()
	}
}

impl BoneKey<EssenceSlot> for AgentDefinition {
	fn bone_key(&self, value: &str) -> Option<EssenceSlot> {
		self.essences.get(value).copied()
	}
}

impl LoadoutConfig for AgentDefinition {
	fn inventory(&self) -> impl Iterator<Item = Option<ItemName>> {
		self.default_inventory_loadout.iter().cloned()
	}

	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<ItemName>)> {
		self.default_slots_loadout.iter().cloned()
	}
}
