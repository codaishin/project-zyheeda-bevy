use crate::{
	tools::bone::Bone,
	traits::{
		accessors::get::GetFromSystemParam,
		loadout::LoadoutConfig,
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
};
use bevy::ecs::component::Component;

pub trait HandlesAgents {
	type TAgentConfig<'a>: LoadoutConfig
		+ Mapper<Bone<'a>, Option<EssenceSlot>>
		+ Mapper<Bone<'a>, Option<HandSlot>>
		+ Mapper<Bone<'a>, Option<ForearmSlot>>;
	type TAgent: Component
		+ for<'i> GetFromSystemParam<AgentConfig, TItem<'i> = Self::TAgentConfig<'i>>;
}

pub struct AgentConfig;
