mod dto;

use crate::{
	behaviors::SkillBehaviorConfig,
	skills::dto::run_skill_behavior::skill_behavior_config::SkillBehaviorConfigDto,
	traits::{Flush, Schedule},
};
use bevy::prelude::*;
use common::{
	self,
	components::persistent_entity::PersistentEntity,
	tools::action_key::slot::SlotKey,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[savable_component(dto = SkillExecuter<SkillBehaviorConfigDto>)]
pub(crate) enum SkillExecuter<TSkillBehavior = SkillBehaviorConfig> {
	#[default]
	Idle,
	Start {
		slot_key: SlotKey,
		shape: TSkillBehavior,
	},
	Stoppable(PersistentEntity),
	Stop(PersistentEntity),
}

impl<TBehavior> Schedule<TBehavior> for SkillExecuter<TBehavior> {
	fn schedule(&mut self, slot_key: SlotKey, behavior: TBehavior) {
		*self = SkillExecuter::Start {
			slot_key,
			shape: behavior,
		};
	}
}

impl<TBehavior> Flush for SkillExecuter<TBehavior> {
	fn flush(&mut self) {
		match self {
			SkillExecuter::Stoppable(entity) => {
				*self = SkillExecuter::Stop(*entity);
			}
			SkillExecuter::Start { .. } => {
				*self = SkillExecuter::Idle;
			}
			_ => {}
		}
	}
}
