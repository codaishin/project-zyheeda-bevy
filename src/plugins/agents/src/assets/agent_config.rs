pub(crate) mod dto;

use crate::systems::insert_default_loadout::internal::GetDefaultLoadout;
use bevy::prelude::*;
use common::{
	tools::{
		action_key::slot::SlotKey,
		bone_name::BoneName,
		inventory_key::InventoryKey,
		path::Path,
	},
	traits::{
		accessors::get::GetProperty,
		handles_animations::{AffectedAnimationBones, Animation, AnimationKey, AnimationMaskBits},
		handles_custom_assets::AssetFolderPath,
		handles_loadout::LoadoutKey,
		handles_physics::PhysicalDefaultAttributes,
		handles_skill_physics::SkillSpawner,
		loadout::ItemName,
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter::Enumerate, slice::Iter};

#[derive(Asset, TypePath, Debug, PartialEq, Default, Clone)]
pub struct AgentConfig {
	pub(crate) loadout: Loadout,
	pub(crate) bones: Bones,
	pub(crate) agent_model: AgentModel,
	pub(crate) attributes: PhysicalDefaultAttributes,
	pub(crate) animations: HashMap<AnimationKey, Animation>,
	pub(crate) animation_mask_groups: HashMap<AnimationMaskBits, AffectedAnimationBones>,
}

impl AssetFolderPath for AgentConfig {
	fn asset_folder_path() -> Path {
		Path::from("agents")
	}
}

impl GetProperty<PhysicalDefaultAttributes> for AgentConfig {
	fn get_property(&self) -> PhysicalDefaultAttributes {
		self.attributes
	}
}

impl GetDefaultLoadout for AgentConfig {
	type TLoadout<'a>
		= LoadoutIterator<'a>
	where
		Self: 'a;

	fn get_default_loadout(&self) -> Self::TLoadout<'_> {
		LoadoutIterator {
			inventory: self.loadout.inventory.iter().enumerate(),
			slots: self.loadout.slots.iter(),
		}
	}
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Loadout {
	inventory: Vec<Option<ItemName>>,
	slots: Vec<(SlotKey, Option<ItemName>)>,
}

pub struct LoadoutIterator<'a> {
	inventory: Enumerate<Iter<'a, Option<ItemName>>>,
	slots: Iter<'a, (SlotKey, Option<ItemName>)>,
}

impl LoadoutIterator<'_> {
	fn inventory_next(&mut self) -> Option<(LoadoutKey, Option<ItemName>)> {
		self.inventory
			.next()
			.map(|(key, item)| (LoadoutKey::Inventory(InventoryKey(key)), item.clone()))
	}

	fn slot_next(&mut self) -> Option<(LoadoutKey, Option<ItemName>)> {
		self.slots
			.next()
			.map(|(key, item)| (LoadoutKey::Slot(*key), item.clone()))
	}
}

impl Iterator for LoadoutIterator<'_> {
	type Item = (LoadoutKey, Option<ItemName>);

	fn next(&mut self) -> Option<Self::Item> {
		self.inventory_next().or_else(|| self.slot_next())
	}
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Bones {
	pub(crate) spawners: HashMap<BoneName, SkillSpawner>,
	pub(crate) hand_slots: HashMap<BoneName, SlotKey>,
	pub(crate) forearm_slots: HashMap<BoneName, SlotKey>,
	pub(crate) essence_slots: HashMap<BoneName, SlotKey>,
}

#[derive(Debug, Clone)]
pub(crate) enum AgentModel {
	Asset(String),
	Procedural(fn(&mut ZyheedaEntityCommands)),
}

impl From<&str> for AgentModel {
	fn from(value: &str) -> Self {
		Self::Asset(String::from(value))
	}
}

impl Default for AgentModel {
	fn default() -> Self {
		Self::Procedural(|_| {})
	}
}

impl PartialEq for AgentModel {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Asset(l0), Self::Asset(r0)) => l0 == r0,
			(Self::Procedural(l0), Self::Procedural(r0)) => std::ptr::fn_addr_eq(*l0, *r0),
			_ => false,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use LoadoutKey::{Inventory, Slot};
	use common::tools::inventory_key::InventoryKey;

	#[test]
	fn iter_inventory_items() {
		let config = AgentConfig {
			loadout: Loadout {
				inventory: vec![Some(ItemName::from("a")), None, Some(ItemName::from("b"))],
				slots: vec![],
			},
			..default()
		};

		let default_loadout = config.get_default_loadout().collect::<Vec<_>>();

		assert_eq!(
			vec![
				(Inventory(InventoryKey(0)), Some(ItemName::from("a"))),
				(Inventory(InventoryKey(1)), None),
				(Inventory(InventoryKey(2)), Some(ItemName::from("b")))
			],
			default_loadout,
		);
	}

	#[test]
	fn iter_slot_items() {
		let config = AgentConfig {
			loadout: Loadout {
				inventory: vec![],
				slots: vec![
					(SlotKey(11), Some(ItemName::from("a"))),
					(SlotKey(4), None),
					(SlotKey(42), Some(ItemName::from("b"))),
				],
			},
			..default()
		};

		let default_loadout = config.get_default_loadout().collect::<Vec<_>>();

		assert_eq!(
			vec![
				(Slot(SlotKey(11)), Some(ItemName::from("a"))),
				(Slot(SlotKey(4)), None),
				(Slot(SlotKey(42)), Some(ItemName::from("b")))
			],
			default_loadout,
		);
	}
}
