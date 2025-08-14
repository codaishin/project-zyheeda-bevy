use crate::{
	SkillDto,
	components::queue::{Queue, SkillElapsed, State},
	skills::{QueuedSkill, Skill, SkillMode},
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
	elapsed: Option<SkillElapsed<DurationSecsF32>>,
	state: State,
}

impl From<Queue> for QueueDto {
	fn from(
		Queue {
			queue,
			active: elapsed,
			state,
		}: Queue,
	) -> Self {
		Self {
			queue: queue.into_iter().map(QueuedSkillDto::from).collect(),
			elapsed: elapsed.map(|SkillElapsed { active, released }| SkillElapsed {
				active: DurationSecsF32::from(active),
				released: DurationSecsF32::from(released),
			}),
			state,
		}
	}
}

impl TryLoadFrom<QueueDto> for Queue {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		QueueDto {
			queue,
			elapsed,
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
			active: elapsed.map(|SkillElapsed { active, released }| SkillElapsed {
				active: Duration::from(active),
				released: Duration::from(released),
			}),
			state,
		})
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct QueuedSkillDto {
	skill: SkillDto,
	key: SlotKey,
	skill_mode: SkillMode,
}

impl From<QueuedSkill> for QueuedSkillDto {
	fn from(skill: QueuedSkill) -> Self {
		Self {
			skill: SkillDto::from(skill.skill),
			key: skill.key,
			skill_mode: skill.skill_mode,
		}
	}
}

impl TryLoadFrom<QueuedSkillDto> for QueuedSkill {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		QueuedSkillDto {
			skill,
			key,
			skill_mode: activation,
		}: QueuedSkillDto,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		let Ok(skill) = Skill::try_load_from(skill, asset_server);

		Ok(Self {
			skill,
			key,
			skill_mode: activation,
		})
	}
}
