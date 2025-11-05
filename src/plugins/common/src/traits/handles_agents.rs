use crate::traits::{
	accessors::get::GetFromSystemParam,
	bone_key::ConfiguredBones,
	loadout::LoadoutConfig,
	visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
};
use bevy::ecs::component::Component;

pub trait HandlesAgents {
	type TAgentConfig<'a>: LoadoutConfig
		+ ConfiguredBones<EssenceSlot>
		+ ConfiguredBones<HandSlot>
		+ ConfiguredBones<ForearmSlot>;
	type TAgent: Component
		+ for<'i> GetFromSystemParam<AgentConfig, TItem<'i> = Self::TAgentConfig<'i>>;
}

pub struct AgentConfig;
