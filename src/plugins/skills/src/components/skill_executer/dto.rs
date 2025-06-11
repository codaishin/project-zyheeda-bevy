use crate::{SkillExecuter, skills::dto::run_skill_behavior::RunSkillBehaviorDto};

pub(crate) type SkillExecuterDto = SkillExecuter<RunSkillBehaviorDto>;

impl From<SkillExecuter> for SkillExecuterDto {
	fn from(value: SkillExecuter) -> Self {
		match value {
			SkillExecuter::Idle => Self::Idle,
			SkillExecuter::Start { slot_key, shape } => Self::Start {
				slot_key,
				shape: RunSkillBehaviorDto::from(shape),
			},
			SkillExecuter::StartedStoppable(persistent_entity) => {
				Self::StartedStoppable(persistent_entity)
			}
			SkillExecuter::Stop(persistent_entity) => Self::Stop(persistent_entity),
		}
	}
}
