use crate::{
	components::{
		enemy::void_sphere::{VoidSphere, VoidSphereSlot},
		player::Player,
	},
	systems::load_agent::InsertSpecializedAgent,
};
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
		accessors::get::GetProperty,
		handles_agents::AgentType,
		handles_custom_assets::AssetFileExtensions,
		handles_enemies::EnemyType,
		handles_skill_behaviors::SkillSpawner,
		loadout::LoadoutConfig,
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Asset, TypePath, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AgentConfigAsset {
	pub(crate) agent_type: AgentType,
	pub(crate) loadout: Loadout,
	pub(crate) bones: Bones,
	pub(crate) attributes: Attributes,
}

impl InsertSpecializedAgent for AgentConfigAsset {
	fn insert_specialized_agent(&self, entity: &mut ZyheedaEntityCommands) {
		match self.agent_type {
			AgentType::Player => entity.try_insert(Player),
			AgentType::Enemy(EnemyType::VoidSphere) => entity.try_insert(VoidSphere),
		};
	}
}

#[derive(Debug, PartialEq)]
pub struct AgentConfig<'a, TAsset = AgentConfigAsset> {
	asset: &'a TAsset,
}

impl<'a, TAsset> From<&'a TAsset> for AgentConfig<'a, TAsset> {
	fn from(asset: &'a TAsset) -> Self {
		Self { asset }
	}
}

impl AssetFileExtensions for AgentConfigAsset {
	fn asset_file_extensions() -> &'static [&'static str] {
		&["agent"]
	}
}

impl VisibleSlots for AgentConfig<'_> {
	fn visible_slots(&self) -> impl Iterator<Item = SlotKey> {
		self.asset
			.loadout
			.visible_slots
			.iter()
			.copied()
			.map(SlotKey::from)
	}
}

impl LoadoutConfig for AgentConfig<'_> {
	fn inventory(&self) -> impl Iterator<Item = Option<AssetPath<'static>>> {
		self.asset.loadout.inventory.iter().cloned()
	}

	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<AssetPath<'static>>)> {
		self.asset
			.loadout
			.slots
			.iter()
			.cloned()
			.map(|(key, item)| (SlotKey::from(key), item))
	}
}

impl Mapper<Bone<'_>, Option<SkillSpawner>> for AgentConfig<'_> {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<SkillSpawner> {
		self.asset
			.bones
			.spawners
			.get(bone)
			.copied()
			.map(SkillSpawner::from)
	}
}

impl Mapper<Bone<'_>, Option<EssenceSlot>> for AgentConfig<'_> {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<EssenceSlot> {
		self.asset
			.bones
			.essence_slots
			.get(bone)
			.copied()
			.map(SlotKey::from)
			.map(EssenceSlot::from)
	}
}

impl Mapper<Bone<'_>, Option<HandSlot>> for AgentConfig<'_> {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<HandSlot> {
		self.asset
			.bones
			.hand_slots
			.get(bone)
			.copied()
			.map(SlotKey::from)
			.map(HandSlot::from)
	}
}

impl Mapper<Bone<'_>, Option<ForearmSlot>> for AgentConfig<'_> {
	fn map(&self, Bone(bone): Bone<'_>) -> Option<ForearmSlot> {
		self.asset
			.bones
			.forearm_slots
			.get(bone)
			.copied()
			.map(SlotKey::from)
			.map(ForearmSlot::from)
	}
}

impl GetProperty<AttributeOnSpawn<Health>> for AgentConfig<'_> {
	fn get_property(&self) -> Health {
		self.asset.attributes.health
	}
}

impl GetProperty<AttributeOnSpawn<EffectTarget<Gravity>>> for AgentConfig<'_> {
	fn get_property(&self) -> EffectTarget<Gravity> {
		self.asset.attributes.gravity_interaction
	}
}

impl GetProperty<AttributeOnSpawn<EffectTarget<Force>>> for AgentConfig<'_> {
	fn get_property(&self) -> EffectTarget<Force> {
		self.asset.attributes.force_interaction
	}
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
