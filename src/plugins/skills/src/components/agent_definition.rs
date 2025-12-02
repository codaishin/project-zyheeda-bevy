use bevy::prelude::*;
use common::{
	tools::{action_key::slot::SlotKey, bone_name::BoneName},
	traits::{
		accessors::get::{AssociatedSystemParam, GetFromSystemParam, TryApplyOn},
		bone_key::{BoneKey, ConfiguredBones},
		handles_agents::AgentConfig,
		loadout::{ItemName, LoadoutConfig},
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct AgentDefinition {
	pub(crate) forearms: HashMap<BoneName, ForearmSlot>,
	pub(crate) hands: HashMap<BoneName, HandSlot>,
	pub(crate) essences: HashMap<BoneName, EssenceSlot>,
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

impl AgentDefinition {
	// FIXME: Remove when exposing interface to insert from outside this plugin
	/// Temporary observer to insert definitions from agent
	pub(crate) fn insert_from_agent<TAgent>(
		trigger: Trigger<OnAdd, TAgent>,
		mut commands: ZyheedaCommands,
		agents: Query<&TAgent>,
		p: AssociatedSystemParam<TAgent, AgentConfig>,
	) where
		TAgent: Component + GetFromSystemParam<AgentConfig>,
		for<'i> TAgent::TItem<'i>: LoadoutConfig
			+ ConfiguredBones<ForearmSlot>
			+ ConfiguredBones<HandSlot>
			+ ConfiguredBones<EssenceSlot>,
	{
		let entity = trigger.target();
		let Ok(agent) = agents.get(entity) else {
			return;
		};
		let Some(conf) = agent.get_from_param(&AgentConfig, &p) else {
			return;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(Self {
				forearms: get_bones(&conf),
				hands: get_bones(&conf),
				essences: get_bones(&conf),
				default_inventory_loadout: conf.inventory().collect(),
				default_slots_loadout: conf.slots().collect(),
			});
		});
	}
}

fn get_bones<TKey>(conf: &impl ConfiguredBones<TKey>) -> HashMap<BoneName, TKey> {
	conf.bone_names()
		.filter_map(|bone| {
			let key = conf.bone_key(&bone)?;
			Some((bone.clone(), key))
		})
		.collect()
}
