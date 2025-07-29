use crate::{
	SkillExecuter,
	skills::{RunSkillBehavior, dto::run_skill_behavior::RunSkillBehaviorDto},
};
use common::{
	errors::Unreachable,
	traits::{handles_custom_assets::TryLoadFrom, load_asset::LoadAsset},
};

impl From<SkillExecuter> for SkillExecuter<RunSkillBehaviorDto> {
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

impl TryLoadFrom<SkillExecuter<RunSkillBehaviorDto>> for SkillExecuter {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		value: SkillExecuter<RunSkillBehaviorDto>,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		let executer = match value {
			SkillExecuter::Idle => Self::Idle,
			SkillExecuter::Start { slot_key, shape } => Self::Start {
				slot_key,
				shape: RunSkillBehavior::from(shape),
			},
			SkillExecuter::StartedStoppable(persistent_entity) => {
				Self::StartedStoppable(persistent_entity)
			}
			SkillExecuter::Stop(persistent_entity) => Self::Stop(persistent_entity),
		};

		Ok(executer)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::behaviors::{
		SkillBehaviorConfig,
		spawn_skill::{SpawnSkill, spawn_shield::SpawnShield},
	};
	use bevy::{asset::AssetPath, prelude::*};
	use common::{
		components::persistent_entity::PersistentEntity,
		tools::action_key::slot::{PlayerSlot, Side, SlotKey},
	};
	use test_case::test_case;

	fn start_spawn_shield() -> SkillExecuter {
		SkillExecuter::Start {
			slot_key: SlotKey::from(PlayerSlot::Lower(Side::Left)),
			shape: RunSkillBehavior::OnAim(SkillBehaviorConfig::from_shape(
				SpawnSkill::Shield(SpawnShield),
			)),
		}
	}

	struct _LoadAsset;

	impl LoadAsset for _LoadAsset {
		fn load_asset<TAsset, TPath>(&mut self, _: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			panic!("SHOULD NOT BE USED")
		}
	}

	#[test_case(SkillExecuter::Idle; "idle")]
	#[test_case(start_spawn_shield(); "start")]
	#[test_case(SkillExecuter::StartedStoppable(PersistentEntity::default()); "started stoppable")]
	#[test_case(SkillExecuter::Stop(PersistentEntity::default()); "stop")]
	fn roundtrip_survives(original: SkillExecuter) {
		let dto = SkillExecuter::<RunSkillBehaviorDto>::from(original.clone());
		let Ok(restored) = SkillExecuter::try_load_from(dto, &mut _LoadAsset);
		assert_eq!(original, restored);
	}
}
