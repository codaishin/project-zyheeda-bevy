use crate::components::enemy::void_sphere::VoidSphereSlot;
use bevy::{
	asset::{Asset, AssetPath},
	reflect::TypePath,
};
use common::{
	attributes::{effect_target::EffectTarget, health::Health},
	effects::{force::Force, gravity::Gravity},
	tools::{
		action_key::slot::{PlayerSlot, SlotKey},
		attribute::AttributeOnSpawn,
		bone::Bone,
	},
	traits::{
		handles_custom_assets::AssetFileExtensions,
		handles_skill_behaviors::SkillSpawner,
		loadout::LoadoutConfig,
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Asset, TypePath, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct AgentAsset {
	pub(crate) loadout: Loadout,
	pub(crate) bones: Bones,
	pub(crate) attributes: Attributes,
}

impl AssetFileExtensions for AgentAsset {
	fn asset_file_extensions() -> &'static [&'static str] {
		&["agent"]
	}
}

impl VisibleSlots for AgentAsset {
	fn visible_slots(&self) -> impl Iterator<Item = SlotKey> {
		self.loadout
			.visible_slots
			.iter()
			.copied()
			.map(SlotKey::from)
	}
}

impl LoadoutConfig for AgentAsset {
	fn inventory(&self) -> impl Iterator<Item = Option<AssetPath<'static>>> {
		self.loadout.inventory.iter().cloned()
	}

	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<AssetPath<'static>>)> {
		self.loadout
			.slots
			.iter()
			.cloned()
			.map(|(key, item)| (SlotKey::from(key), item))
	}
}

impl Mapper<Bone<'_>, Option<SkillSpawner>> for AgentAsset {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<SkillSpawner> {
		self.bones
			.spawners
			.get(bone)
			.copied()
			.map(SkillSpawner::from)
	}
}

impl Mapper<Bone<'_>, Option<EssenceSlot>> for AgentAsset {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<EssenceSlot> {
		self.bones
			.essence_slots
			.get(bone)
			.copied()
			.map(SlotKey::from)
			.map(EssenceSlot::from)
	}
}

impl Mapper<Bone<'_>, Option<HandSlot>> for AgentAsset {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<HandSlot> {
		self.bones
			.hand_slots
			.get(bone)
			.copied()
			.map(SlotKey::from)
			.map(HandSlot::from)
	}
}

impl Mapper<Bone<'_>, Option<ForearmSlot>> for AgentAsset {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<ForearmSlot> {
		self.bones
			.forearm_slots
			.get(bone)
			.copied()
			.map(SlotKey::from)
			.map(ForearmSlot::from)
	}
}

impl From<&AgentAsset> for AttributeOnSpawn<Health> {
	fn from(agent: &AgentAsset) -> Self {
		Self(agent.attributes.health)
	}
}

impl From<&AgentAsset> for AttributeOnSpawn<EffectTarget<Gravity>> {
	fn from(agent: &AgentAsset) -> Self {
		Self(agent.attributes.gravity_interaction)
	}
}

impl From<&AgentAsset> for AttributeOnSpawn<EffectTarget<Force>> {
	fn from(agent: &AgentAsset) -> Self {
		Self(agent.attributes.force_interaction)
	}
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Loadout {
	visible_slots: Vec<AgentSlotKey>,
	inventory: Vec<Option<AssetPath<'static>>>,
	slots: Vec<(AgentSlotKey, Option<AssetPath<'static>>)>,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum AgentSlotKey {
	Player(PlayerSlot),
	VoidSphere(VoidSphereSlot),
}

impl From<AgentSlotKey> for SlotKey {
	fn from(key: AgentSlotKey) -> Self {
		match key {
			AgentSlotKey::Player(key) => Self::from(key),
			AgentSlotKey::VoidSphere(key) => Self::from(key),
		}
	}
}

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Bones {
	spawners: HashMap<String, SkillSpawnerDto>,
	hand_slots: HashMap<String, AgentSlotKey>,
	forearm_slots: HashMap<String, AgentSlotKey>,
	essence_slots: HashMap<String, AgentSlotKey>,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum SkillSpawnerDto {
	Neutral,
	Slot(AgentSlotKey),
}

impl From<SkillSpawnerDto> for SkillSpawner {
	fn from(value: SkillSpawnerDto) -> Self {
		match value {
			SkillSpawnerDto::Neutral => Self::Neutral,
			SkillSpawnerDto::Slot(key) => Self::Slot(SlotKey::from(key)),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]

pub(crate) struct Attributes {
	health: Health,
	gravity_interaction: EffectTarget<Gravity>,
	force_interaction: EffectTarget<Force>,
}

#[cfg(test)]
impl Default for Attributes {
	fn default() -> Self {
		Self {
			health: Health::new(100.),
			gravity_interaction: EffectTarget::Affected,
			force_interaction: EffectTarget::Affected,
		}
	}
}
