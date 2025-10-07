use crate::{
	tools::{action_key::slot::SlotKey, bone::Bone},
	traits::{
		accessors::get::GetFromSystemParam,
		handles_enemies::EnemyType,
		handles_skill_behaviors::SkillSpawner,
		loadout::LoadoutConfig,
		mapper::Mapper,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
	zyheeda_commands::ZyheedaCommands,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub trait HandlesAgents {
	type TAgentConfig<'a>: LoadoutConfig
		+ Mapper<Bone<'a>, Option<SkillSpawner>>
		+ Mapper<Bone<'a>, Option<EssenceSlot>>
		+ Mapper<Bone<'a>, Option<HandSlot>>
		+ Mapper<Bone<'a>, Option<ForearmSlot>>;
	type TAgent: Component
		+ Spawn
		+ for<'i> GetFromSystemParam<CurrentAction, TItem<'i> = AgentActionTarget>
		+ for<'i> GetFromSystemParam<AgentConfig, TItem<'i> = Self::TAgentConfig<'i>>;
}

pub trait Spawn {
	fn spawn<'a>(commands: &'a mut ZyheedaCommands, agent_type: AgentType) -> EntityCommands<'a>;
}

pub struct AgentConfig;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum AgentType {
	Player,
	Enemy(EnemyType),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CurrentAction {
	Movement,
	UseSkill(SlotKey),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AgentActionTarget {
	Point(Vec3),
	Direction(Dir3),
	Entity(Entity),
}
