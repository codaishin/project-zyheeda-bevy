pub(crate) mod dto;

use crate::systems::agent::insert_model::InsertModel;
use bevy::prelude::*;
use common::{
	components::asset_model::AssetModel,
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::GetProperty,
		bone_key::BoneKey,
		handles_custom_assets::AssetFolderPath,
		handles_map_generation::AgentType,
		handles_physics::PhysicalDefaultAttributes,
		handles_skill_behaviors::SkillSpawner,
		load_asset::Path,
		loadout::{ItemName, LoadoutConfig},
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Asset, TypePath, Debug, PartialEq, Default, Clone)]
pub struct AgentConfigAsset {
	pub(crate) loadout: Loadout,
	pub(crate) bones: Bones,
	pub(crate) agent_model: AgentModel,
	pub(crate) attributes: PhysicalDefaultAttributes,
}

impl AssetFolderPath for AgentConfigAsset {
	fn asset_folder_path() -> Path {
		Path::from("agents")
	}
}

impl GetProperty<PhysicalDefaultAttributes> for AgentConfigAsset {
	fn get_property(&self) -> PhysicalDefaultAttributes {
		self.attributes
	}
}

#[derive(Debug, PartialEq)]
pub struct AgentConfigData<'a, TAsset = AgentConfigAsset> {
	pub(crate) agent_type: AgentType,
	pub(crate) asset: &'a TAsset,
}

impl LoadoutConfig for AgentConfigData<'_> {
	fn inventory(&self) -> impl Iterator<Item = Option<ItemName>> {
		self.asset.loadout.inventory.iter().cloned()
	}

	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<ItemName>)> {
		self.asset.loadout.slots.iter().cloned()
	}
}

impl BoneKey<EssenceSlot> for AgentConfigData<'_> {
	fn bone_key(&self, bone_name: &str) -> Option<EssenceSlot> {
		self.asset
			.bones
			.essence_slots
			.get(bone_name)
			.copied()
			.map(EssenceSlot::from)
	}
}

impl BoneKey<HandSlot> for AgentConfigData<'_> {
	fn bone_key(&self, bone_name: &str) -> Option<HandSlot> {
		self.asset
			.bones
			.hand_slots
			.get(bone_name)
			.copied()
			.map(HandSlot::from)
	}
}

impl BoneKey<ForearmSlot> for AgentConfigData<'_> {
	fn bone_key(&self, bone_name: &str) -> Option<ForearmSlot> {
		self.asset
			.bones
			.forearm_slots
			.get(bone_name)
			.copied()
			.map(ForearmSlot::from)
	}
}

impl InsertModel for AgentConfigData<'_> {
	fn insert_model(&self, entity: &mut ZyheedaEntityCommands) {
		match &self.asset.agent_model {
			AgentModel::Asset(path) => {
				entity.try_insert(AssetModel::from(path));
			}
			AgentModel::Procedural(insert_procedural_on) => insert_procedural_on(entity),
		}
	}
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Loadout {
	inventory: Vec<Option<ItemName>>,
	slots: Vec<(SlotKey, Option<ItemName>)>,
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Bones {
	pub(crate) spawners: HashMap<String, SkillSpawner>,
	pub(crate) hand_slots: HashMap<String, SlotKey>,
	pub(crate) forearm_slots: HashMap<String, SlotKey>,
	pub(crate) essence_slots: HashMap<String, SlotKey>,
}

#[derive(Debug, Clone)]
pub(crate) enum AgentModel {
	Asset(String),
	Procedural(fn(&mut ZyheedaEntityCommands)),
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
