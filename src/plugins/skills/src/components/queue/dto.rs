use crate::{
	SkillDto,
	components::queue::{Queue, State},
	skills::{Activation, QueuedSkill, Skill},
};
use common::{
	dto::duration_secs_f32::DurationSecsF32,
	errors::Unreachable,
	tools::action_key::slot::SlotKey,
	traits::{handles_custom_assets::TryLoadFrom, load_asset::LoadAsset},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct QueueDto {
	queue: Vec<QueuedSkillDto>,
	#[serde(skip_serializing_if = "Option::is_none")]
	duration: Option<DurationSecsF32>,
	state: State,
}

impl From<Queue> for QueueDto {
	fn from(queue: Queue) -> Self {
		Self {
			queue: queue.queue.into_iter().map(QueuedSkillDto::from).collect(),
			duration: queue.duration.map(DurationSecsF32::from),
			state: queue.state,
		}
	}
}

impl TryLoadFrom<QueueDto> for Queue {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		QueueDto {
			queue,
			duration,
			state,
		}: QueueDto,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		Ok(Self {
			queue: queue
				.into_iter()
				.map(|skill| {
					let Ok(skill) = QueuedSkill::try_load_from(skill, asset_server);
					skill
				})
				.collect(),
			duration: duration.map(Duration::from),
			state,
		})
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

impl TryLoadFrom<QueuedSkillDto> for QueuedSkill {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		QueuedSkillDto {
			skill,
			slot_key,
			mode,
		}: QueuedSkillDto,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		let Ok(skill) = Skill::try_load_from(skill, asset_server);

		Ok(Self {
			skill,
			slot_key,
			mode,
		})
	}
}
