use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{AssociatedSystemParam, GetFromSystemParam, TryApplyOn},
		bone_key::BoneKey,
		handles_agents::AgentConfig,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct SlotsDefinition {
	pub(crate) forearms: HashMap<String, ForearmSlot>,
	pub(crate) hands: HashMap<String, HandSlot>,
	pub(crate) essences: HashMap<String, EssenceSlot>,
}

impl BoneKey<ForearmSlot> for SlotsDefinition {
	fn bone_key(&self, value: &str) -> Option<ForearmSlot> {
		self.forearms.get(value).copied()
	}
}

impl BoneKey<HandSlot> for SlotsDefinition {
	fn bone_key(&self, value: &str) -> Option<HandSlot> {
		self.hands.get(value).copied()
	}
}

impl BoneKey<EssenceSlot> for SlotsDefinition {
	fn bone_key(&self, value: &str) -> Option<EssenceSlot> {
		self.essences.get(value).copied()
	}
}

impl SlotsDefinition {
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
		for<'i> TAgent::TItem<'i>: BoneKey<ForearmSlot>,
		for<'i> TAgent::TItem<'i>: BoneKey<HandSlot>,
		for<'i> TAgent::TItem<'i>: BoneKey<EssenceSlot>,
	{
		let entity = trigger.target();
		let Ok(agent) = agents.get(entity) else {
			return;
		};
		let Some(item) = agent.get_from_param(&AgentConfig, &p) else {
			return;
		};

		commands.try_apply_on(&entity, |mut e| {});
	}
}
