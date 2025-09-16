use crate::{
	attributes::{effect_target::EffectTarget, health::Health},
	effects::{force::Force, gravity::Gravity},
	tools::{attribute::AttributeOnSpawn, bone::Bone},
	traits::{
		accessors::get::{GetFromSystemParam, GetProperty},
		handles_enemies::EnemyType,
		handles_skill_behaviors::SkillSpawner,
		loadout::LoadoutConfig,
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};

pub trait HandlesAgents {
	type TAgentData<'a>: VisibleSlots
		+ LoadoutConfig
		+ Mapper<Bone<'a>, Option<SkillSpawner>>
		+ Mapper<Bone<'a>, Option<EssenceSlot>>
		+ Mapper<Bone<'a>, Option<HandSlot>>
		+ Mapper<Bone<'a>, Option<ForearmSlot>>
		+ GetProperty<AttributeOnSpawn<Health>>
		+ GetProperty<AttributeOnSpawn<EffectTarget<Gravity>>>
		+ GetProperty<AttributeOnSpawn<EffectTarget<Force>>>;
	type TAgent: Component
		+ From<AgentType>
		+ for<'i> GetFromSystemParam<(), TItem<'i> = Self::TAgentData<'i>>;
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum AgentType {
	Player,
	Enemy(EnemyType),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AgentAssetNotLoaded;
