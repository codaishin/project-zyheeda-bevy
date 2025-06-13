use crate::{
	SkillExecuter,
	skills::{RunSkillBehavior, dto::run_skill_behavior::RunSkillBehaviorDto},
};
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

impl From<SkillExecuterDto> for SkillExecuter {
	fn from(value: SkillExecuterDto) -> Self {
		match value {
			SkillExecuterDto::Idle => Self::Idle,
			SkillExecuterDto::Start { slot_key, shape } => Self::Start {
				slot_key,
				shape: RunSkillBehavior::from(shape),
			},
			SkillExecuterDto::StartedStoppable(persistent_entity) => {
				Self::StartedStoppable(persistent_entity)
			}
			SkillExecuterDto::Stop(persistent_entity) => Self::Stop(persistent_entity),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::behaviors::{
		SkillBehaviorConfig,
		build_skill_shape::{BuildSkillShape, spawn_shield::SpawnShield},
	};
	use common::{
		components::persistent_entity::PersistentEntity,
		tools::action_key::slot::{Side, SlotKey},
	};
	use test_case::test_case;
	fn start_spawn_shield() -> SkillExecuter {
		SkillExecuter::Start {
			slot_key: SlotKey::BottomHand(Side::Left),
			shape: RunSkillBehavior::OnAim(SkillBehaviorConfig::from_shape(
				BuildSkillShape::Shield(SpawnShield),
			)),
		}
	}

	#[test_case(SkillExecuter::Idle; "idle")]
	#[test_case(start_spawn_shield(); "start")]
	#[test_case(SkillExecuter::StartedStoppable(PersistentEntity::default()); "started stoppable")]
	#[test_case(SkillExecuter::Stop(PersistentEntity::default()); "stop")]
	fn roundtrip_survives(original: SkillExecuter) {
		let dto = SkillExecuterDto::from(original.clone());
		let restored = SkillExecuter::from(dto);
		assert_eq!(original, restored);
	}
}
