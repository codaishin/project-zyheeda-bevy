mod dto;

use crate::{
	skills::behaviors::{SkillBehaviorConfig, dto::SkillBehaviorConfigDto},
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
#[savable_component(id = "active skill", dto = ActiveSkill<SkillBehaviorConfigDto>)]
pub(crate) enum ActiveSkill<TSkillBehavior = SkillBehaviorConfig> {
	#[default]
	Idle,
	Start {
		slot_key: SlotKey,
		shape: TSkillBehavior,
	},
	Stoppable(PersistentEntity),
	Stop(PersistentEntity),
}

impl<TBehavior> Schedule<TBehavior> for ActiveSkill<TBehavior> {
	fn schedule(&mut self, slot_key: SlotKey, behavior: TBehavior) {
		*self = ActiveSkill::Start {
			slot_key,
			shape: behavior,
		};
	}
}

impl<TBehavior> Flush for ActiveSkill<TBehavior> {
	fn flush(&mut self) {
		match self {
			ActiveSkill::Stoppable(entity) => {
				*self = ActiveSkill::Stop(*entity);
			}
			ActiveSkill::Start { .. } => {
				*self = ActiveSkill::Idle;
			}
			_ => {}
		}
	}
}
