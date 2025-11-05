use crate::traits::{
	accessors::get::GetFromSystemParam,
	bone_key::BoneKey,
	loadout::LoadoutConfig,
	visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
};
use bevy::ecs::component::Component;

pub trait HandlesAgents {
	type TAgentConfig<'a>: LoadoutConfig
		+ BoneKey<EssenceSlot>
		+ BoneKey<HandSlot>
		+ BoneKey<ForearmSlot>;
	type TAgent: Component
		+ for<'i> GetFromSystemParam<AgentConfig, TItem<'i> = Self::TAgentConfig<'i>>;
}

pub struct AgentConfig;
