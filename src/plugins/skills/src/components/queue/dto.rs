use crate::{
	SkillDto,
	components::queue::{Queue, State},
	skills::{Activation, QueuedSkill},
};
use common::tools::action_key::slot::SlotKey;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct QueueDto {
	queue: Vec<QueuedSkillDto>,
	duration: Option<Duration>,
	state: State,
}

impl From<Queue> for QueueDto {
	fn from(queue: Queue) -> Self {
		Self {
			queue: queue.queue.into_iter().map(QueuedSkillDto::from).collect(),
			duration: queue.duration,
			state: queue.state,
		}
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct QueuedSkillDto {
	skill: SkillDto,
	slot_key: SlotKey,
	mode: Activation,
}

impl From<QueuedSkill> for QueuedSkillDto {
	fn from(skill: QueuedSkill) -> Self {
		Self {
			skill: SkillDto::from(skill.skill),
			slot_key: skill.slot_key,
			mode: skill.mode,
		}
	}
}
