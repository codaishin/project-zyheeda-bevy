pub(crate) mod dto;

use crate::systems::agent::insert_model::InsertModel;
use bevy::{asset::Asset, reflect::TypePath};
use common::{
	attributes::{effect_target::EffectTarget, health::Health},
	components::asset_model::AssetModel,
	effects::{force::Force, gravity::Gravity},
	tools::{action_key::slot::SlotKey, attribute::AttributeOnSpawn, bone::Bone},
	traits::{
		accessors::get::GetProperty,
		handles_agents::AgentType,
		handles_custom_assets::AssetFolderPath,
		handles_physics::PhysicalDefaultAttributes,
		handles_skill_behaviors::SkillSpawner,
		load_asset::Path,
		loadout::{ItemName, LoadoutConfig},
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Asset, TypePath, Debug, PartialEq, Clone)]
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

impl Mapper<Bone<'_>, Option<SkillSpawner>> for AgentConfigData<'_> {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<SkillSpawner> {
		self.asset.bones.spawners.get(bone).copied()
	}
}

impl Mapper<Bone<'_>, Option<EssenceSlot>> for AgentConfigData<'_> {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<EssenceSlot> {
		self.asset
			.bones
			.essence_slots
			.get(bone)
			.copied()
			.map(EssenceSlot::from)
	}
}

impl Mapper<Bone<'_>, Option<HandSlot>> for AgentConfigData<'_> {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<HandSlot> {
		self.asset
			.bones
			.hand_slots
			.get(bone)
			.copied()
			.map(HandSlot::from)
	}
}

impl Mapper<Bone<'_>, Option<ForearmSlot>> for AgentConfigData<'_> {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<ForearmSlot> {
		self.asset
			.bones
			.forearm_slots
			.get(bone)
			.copied()
			.map(ForearmSlot::from)
	}
}

impl GetProperty<AttributeOnSpawn<Health>> for AgentConfigData<'_> {
	fn get_property(&self) -> Health {
		self.asset.attributes.health
	}
}

impl GetProperty<AttributeOnSpawn<EffectTarget<Gravity>>> for AgentConfigData<'_> {
	fn get_property(&self) -> EffectTarget<Gravity> {
		self.asset.attributes.gravity_interaction
	}
}

impl GetProperty<AttributeOnSpawn<EffectTarget<Force>>> for AgentConfigData<'_> {
	fn get_property(&self) -> EffectTarget<Force> {
		self.asset.attributes.force_interaction
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct Loadout {
	inventory: Vec<Option<ItemName>>,
	slots: Vec<(SlotKey, Option<ItemName>)>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

impl PartialEq for AgentModel {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Asset(l0), Self::Asset(r0)) => l0 == r0,
			(Self::Procedural(l0), Self::Procedural(r0)) => std::ptr::fn_addr_eq(*l0, *r0),
			_ => false,
		}
	}
}
