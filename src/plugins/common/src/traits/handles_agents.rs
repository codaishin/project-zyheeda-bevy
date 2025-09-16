use crate::{
	attributes::{effect_target::EffectTarget, health::Health},
	effects::{force::Force, gravity::Gravity},
	tools::{attribute::AttributeOnSpawn, bone::Bone},
	traits::{
		accessors::get::RefInto,
		handles_enemies::EnemyType,
		handles_skill_behaviors::SkillSpawner,
		loadout::LoadoutConfig,
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
	},
};
use bevy::{
	asset::{Asset, Handle},
	ecs::component::Component,
};
use serde::{Deserialize, Serialize};

pub trait HandlesAgents {
	type TAgentAsset: Asset
		+ VisibleSlots
		+ LoadoutConfig
		+ for<'a> Mapper<Bone<'a>, Option<SkillSpawner>>
		+ for<'a> Mapper<Bone<'a>, Option<EssenceSlot>>
		+ for<'a> Mapper<Bone<'a>, Option<HandSlot>>
		+ for<'a> Mapper<Bone<'a>, Option<ForearmSlot>>
		+ for<'a> RefInto<'a, AttributeOnSpawn<Health>>
		+ for<'a> RefInto<'a, AttributeOnSpawn<EffectTarget<Gravity>>>
		+ for<'a> RefInto<'a, AttributeOnSpawn<EffectTarget<Force>>>;
	type TAgent: Component
		+ From<AgentType>
		+ for<'a> RefInto<'a, Result<&'a Handle<Self::TAgentAsset>, AgentNotLoaded>>;
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum AgentType {
	Player,
	Enemy(EnemyType),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AgentNotLoaded;
