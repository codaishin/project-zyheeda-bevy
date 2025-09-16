use crate::{
	attributes::{effect_target::EffectTarget, health::Health},
	effects::{force::Force, gravity::Gravity},
	tools::{attribute::AttributeOnSpawn, bone::Bone},
	traits::{
		accessors::get::{GetFromSystemParam, RefInto},
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
		+ RefInto<'a, AttributeOnSpawn<Health>>
		+ RefInto<'a, AttributeOnSpawn<EffectTarget<Gravity>>>
		+ RefInto<'a, AttributeOnSpawn<EffectTarget<Force>>>;
	type TAgent: Component
		+ From<AgentType>
		+ for<'w, 's, 'i> GetFromSystemParam<'w, 's, (), TItem<'i> = Self::TAgentData<'i>>;
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum AgentType {
	Player,
	Enemy(EnemyType),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AgentAssetNotLoaded;
