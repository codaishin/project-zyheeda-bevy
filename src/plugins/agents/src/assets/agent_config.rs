pub(crate) mod dto;

use bevy::prelude::*;
use common::{
	tools::{action_key::slot::SlotKey, bone_name::BoneName, path::Path},
	traits::{
		accessors::get::GetProperty,
		handles_animations::{AffectedAnimationBones, Animation, AnimationKey, AnimationMaskBits},
		handles_custom_assets::AssetFolderPath,
		handles_physics::PhysicalDefaultAttributes,
		handles_skill_physics::SkillSpawner,
		loadout::ItemName,
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
	pub(crate) ground_offset: Vec3,
	pub(crate) attributes: PhysicalDefaultAttributes,
	pub(crate) animations: HashMap<AnimationKey, Animation>,
	pub(crate) animation_mask_groups: HashMap<AnimationMaskBits, AffectedAnimationBones>,
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

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Loadout {
	pub(crate) inventory: Vec<Option<ItemName>>,
	pub(crate) slots: Vec<(SlotKey, Option<ItemName>)>,
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
