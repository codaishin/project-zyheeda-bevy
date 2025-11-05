use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{AssociatedSystemParam, GetFromSystemParam, TryApplyOn},
		bone_key::{BoneKey, ConfiguredBones},
		handles_agents::AgentConfig,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct BoneDefinition {
	pub(crate) forearms: HashMap<String, ForearmSlot>,
	pub(crate) hands: HashMap<String, HandSlot>,
	pub(crate) essences: HashMap<String, EssenceSlot>,
}

impl BoneKey<ForearmSlot> for BoneDefinition {
	fn bone_key(&self, value: &str) -> Option<ForearmSlot> {
		self.forearms.get(value).copied()
	}
}

impl BoneKey<HandSlot> for BoneDefinition {
	fn bone_key(&self, value: &str) -> Option<HandSlot> {
		self.hands.get(value).copied()
	}
}

impl BoneKey<EssenceSlot> for BoneDefinition {
	fn bone_key(&self, value: &str) -> Option<EssenceSlot> {
		self.essences.get(value).copied()
	}
}

impl BoneDefinition {
	// FIXME: Remove when exposing interface to insert
	//        from outside this plugin
	/// Temporary observer to insert definitions from agent
	pub(crate) fn insert_from_agent<TAgent>(
		trigger: Trigger<OnAdd, TAgent>,
		mut commands: ZyheedaCommands,
		agents: Query<&TAgent>,
		p: AssociatedSystemParam<TAgent, AgentConfig>,
	) where
		TAgent: Component + GetFromSystemParam<AgentConfig>,
		for<'i> TAgent::TItem<'i>: ConfiguredBones<ForearmSlot>,
		for<'i> TAgent::TItem<'i>: ConfiguredBones<HandSlot>,
		for<'i> TAgent::TItem<'i>: ConfiguredBones<EssenceSlot>,
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
			});
		});
	}
}

fn get_bones<TKey>(conf: &impl ConfiguredBones<TKey>) -> HashMap<String, TKey> {
	conf.bone_names()
		.filter_map(|bone| {
			let key = conf.bone_key(bone)?;
			Some((bone.to_owned(), key))
		})
		.collect()
}
